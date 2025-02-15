use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use photondb::{
    env::Env, raw::Table, ChecksumType, Compression, FlushOptions, TableOptions, TableStats,
};

use super::Store;
use crate::bench::{Args, Result};

#[derive(Clone)]
pub(crate) struct PhotondbStore<E: Env + Sync + Send + 'static> {
    table: Table<E>,
}

#[async_trait]
impl<E: Env + Sync + Send + 'static> Store<E> for PhotondbStore<E>
where
    <E as photondb::env::Env>::JoinHandle<()>: Sync,
{
    async fn open_table(config: Arc<Args>, env: &E) -> Self {
        if config.use_existing_db != 1 && config.db.exists() {
            std::fs::remove_dir_all(&config.db).unwrap();
        }
        let mut options = TableOptions::default();
        options.page_store.prepopulate_cache_on_flush = false;
        options.page_store.cache_strict_capacity_limit = true;
        options.page_store.cache_estimated_entry_charge = 4840;
        options.page_store.cache_capacity = config.cache_size as usize;
        options.page_store.max_write_buffers = config.max_write_buffers;
        options.page_store.write_buffer_capacity = config.write_buffer_size as u32;
        options.page_store.disable_space_reclaiming = config.disable_space_reclaiming;
        options.page_store.max_space_amplification_percent = config.max_space_amplification_percent;
        options.page_store.space_used_high = config.space_used_high;
        options.page_size = config.page_size as usize;
        options.page_store.page_checksum_type = if config.verify_checksum == 1 {
            ChecksumType::CRC32
        } else {
            ChecksumType::NONE
        };
        if !config.enable_compression {
            options.page_store.compression_on_cold_compact = Compression::NONE;
            options.page_store.compression_on_flush = Compression::NONE;
        } else {
            options.page_store.compression_on_cold_compact = Compression::SNAPPY;
            options.page_store.compression_on_flush = Compression::ZSTD;
        }
        let table = Table::open(env.to_owned(), &config.db, options)
            .await
            .expect("open table fail");
        Self { table }
    }

    async fn put(&self, key: &[u8], lsn: u64, value: &[u8]) -> Result<()> {
        photondb::perf::reset_perf_ctx();
        self.table.put(key, lsn, value).await.expect("put fail");
        photondb::perf::with(|ctx| {
            if ctx.total > Duration::from_millis(300) {
                log::info!("slow PUT: {ctx:?}");
            }
        });
        Ok(())
    }

    async fn get(&self, key: &[u8], lsn: u64) -> Result<Option<Vec<u8>>> {
        photondb::perf::reset_perf_ctx();
        let r = self.table.get(key, lsn).await.expect("get fail");
        photondb::perf::with(|ctx| {
            if ctx.total > Duration::from_millis(200) {
                log::info!("slow GET: {ctx:?}");
            }
        });
        Ok(r)
    }

    async fn flush(&self) {
        self.table.flush(&FlushOptions::default()).await;
    }

    async fn wait_for_reclaiming(&self) {
        self.table.wait_for_reclaiming().await;
    }

    async fn close(self) -> Result<(), Self> {
        self.table
            .close()
            .await
            .map_err(|table| PhotondbStore { table })
    }

    fn stats(&self) -> Option<TableStats> {
        Some(self.table.stats())
    }
}

impl<E: Env + Sync + Send + 'static> std::fmt::Debug for PhotondbStore<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhotonDbStore").finish()
    }
}
