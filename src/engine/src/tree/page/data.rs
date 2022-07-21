use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    mem::size_of,
};

use super::format::*;

#[derive(Copy, Clone, Debug)]
pub struct Key<'a> {
    pub raw: &'a [u8],
    pub lsn: u64,
}

impl<'a> Key<'a> {
    pub fn new(raw: &'a [u8], lsn: u64) -> Self {
        Self { raw, lsn }
    }
}

impl Eq for Key<'_> {}

impl PartialEq for Key<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw && self.lsn == other.lsn
    }
}

impl Ord for Key<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.raw.cmp(other.raw) {
            Ordering::Equal => self.lsn.cmp(&other.lsn).reverse(),
            o => o,
        }
    }
}

impl PartialOrd for Key<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Encodable for Key<'_> {
    fn encode_to(&self, w: &mut BufWriter) {
        w.put_length_prefixed_slice(self.raw);
        w.put_u64(self.lsn);
    }

    fn encode_size(&self) -> usize {
        BufWriter::length_prefixed_slice_size(self.raw) + size_of::<u64>()
    }
}

impl Decodable for Key<'_> {
    fn decode_from(r: &mut BufReader) -> Self {
        let raw = r.get_length_prefixed_slice();
        let lsn = r.get_u64();
        Self { raw, lsn }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value<'a> {
    Put(&'a [u8]),
    Delete,
}

impl Encodable for Value<'_> {
    fn encode_to(&self, w: &mut BufWriter) {
        match self {
            Value::Put(value) => {
                w.put_u8(ValueKind::Put as u8);
                w.put_length_prefixed_slice(value);
            }
            Value::Delete => w.put_u8(ValueKind::Delete as u8),
        }
    }

    fn encode_size(&self) -> usize {
        1 + match self {
            Value::Put(value) => BufWriter::length_prefixed_slice_size(value),
            Value::Delete => 0,
        }
    }
}

impl Decodable for Value<'_> {
    fn decode_from(r: &mut BufReader) -> Self {
        let kind: ValueKind = r.get_u8().into();
        match kind {
            ValueKind::Put => {
                let value = r.get_length_prefixed_slice();
                Self::Put(value)
            }
            ValueKind::Delete => Self::Delete,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ValueKind {
    Put = 0,
    Delete = 1,
}

impl From<u8> for ValueKind {
    fn from(kind: u8) -> Self {
        match kind {
            0 => Self::Put,
            1 => Self::Delete,
            _ => panic!("invalid data kind"),
        }
    }
}

pub type DataPageBuf = SortedPageBuf;
pub type DataPageBuilder = SortedPageBuilder;
pub type DataPageRef<'a> = SortedPageRef<'a, Key<'a>, Value<'a>>;
pub type DataPageIter<'a> = SortedPageIter<'a, Key<'a>, Value<'a>>;
