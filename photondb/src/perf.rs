use std::{cell::RefCell, ops::DerefMut, time::Duration};

thread_local! {
    static PERF_CTX: RefCell<PerfCtx>  = RefCell::new(Default::default());
}

#[derive(Default, Debug)]
pub struct PerfCtx {
    pub total: Duration,
    pub find_leaf: Duration,
    pub find_value: Duration,
    pub write_build_page: Duration,
    pub replace_page: Duration,
    pub collect_info: Duration,
    pub get_page_info: Duration,
    pub get_page: Duration,
    pub consolidate_page: Duration,
    pub split_page: Duration,
    pub get_page_from_cache_count: u64,
    pub get_page_from_cache_miss_count: u64,
    pub get_page_info_count: u64,
    pub consolidate_page_size: usize,
    pub consolidate_length: usize,
}

pub fn with<F, R>(f: F) -> R
where
    F: FnOnce(&mut PerfCtx) -> R,
{
    PERF_CTX.with(|cell| f(cell.borrow_mut().deref_mut()))
}

pub fn reset_perf_ctx() {
    PERF_CTX.with(|cell| {
        cell.borrow_mut().deref_mut().reset();
    })
}

impl PerfCtx {
    fn reset(&mut self) {
        self.total = Duration::ZERO;
        self.find_leaf = Duration::ZERO;
        self.find_value = Duration::ZERO;
        self.write_build_page = Duration::ZERO;
        self.replace_page = Duration::ZERO;
        self.collect_info = Duration::ZERO;
        self.get_page = Duration::ZERO;
        self.get_page_info = Duration::ZERO;
        self.consolidate_page = Duration::ZERO;
        self.split_page = Duration::ZERO;
        self.get_page_from_cache_count = 0;
        self.get_page_from_cache_miss_count = 0;
        self.get_page_info_count = 0;
        self.consolidate_page_size = 0;
        self.consolidate_length = 0;
    }
}

macro_rules! set_field {
    ($name:ident, $field:ident, $ty:ty) => {
        impl PerfCtx {
            pub(crate) fn $name(self: &mut PerfCtx, value: $ty) {
                self.$field = value;
            }
        }
    };
}

macro_rules! add_field {
    ($name:ident, $field:ident, $ty:ty) => {
        impl PerfCtx {
            pub(crate) fn $name(&mut self, value: $ty) {
                self.$field = self.$field.saturating_add(value);
            }
        }
    };
}

set_field!(set_find_leaf, find_leaf, Duration);
set_field!(set_find_value, find_value, Duration);
set_field!(set_write_build_page, write_build_page, Duration);
set_field!(set_total, total, Duration);

add_field!(add_consolidate_page, consolidate_page, Duration);
add_field!(add_collect_info, collect_info, Duration);
add_field!(add_get_page, get_page, Duration);
add_field!(add_get_page_info, get_page_info, Duration);
add_field!(add_replace_page, replace_page, Duration);
add_field!(add_split_page, split_page, Duration);

impl PerfCtx {
    pub(crate) fn inc_get_page_from_cache_miss_count(&mut self) {
        self.get_page_from_cache_miss_count += 1;
    }
    pub(crate) fn inc_get_page_from_cache_count(&mut self) {
        self.get_page_from_cache_count += 1;
    }
    pub(crate) fn inc_get_page_info_count(&mut self) {
        self.get_page_info_count += 1;
    }
    pub(crate) fn add_consolidate_page_size(&mut self, size: usize) {
        self.consolidate_page_size += size;
    }
    pub(crate) fn add_consolidate_length(&mut self, len: usize) {
        self.consolidate_length += len;
    }
}
