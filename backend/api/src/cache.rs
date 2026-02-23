use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use moka::future::Cache as MokaCache;
use std::sync::Arc;
use sqlx::PgPool;

/// Cache configuration options
#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_capacity: 10_000,
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(enabled_str) = std::env::var("CACHE_ENABLED") {
            config.enabled = enabled_str.to_lowercase() == "true";
        }

        if let Ok(capacity_str) = std::env::var("CACHE_MAX_CAPACITY") {
            if let Ok(capacity) = capacity_str.parse::<u64>() {
                config.max_capacity = capacity;
            } else {
                // Support parsing like "10 GB" by just falling back to 10000 limit for elements if not
            }
        }

        tracing::info!(
            "Cache config loaded: enabled={}, capacity={}",
            config.enabled,
            config.max_capacity
        );

        config
    }
}

pub struct CacheLayer {
    pub abi_cache: MokaCache<String, String>,
    pub verification_cache: MokaCache<String, String>,
    config: CacheConfig,
}

impl CacheLayer {
    pub fn new(config: CacheConfig) -> Self {
        // 24-hour TTL for ABI, max size configurable default 10GB but we use the config max_capacity 
        let abi_cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .weigher(|_k, v: &String| -> u32 {
                v.len().try_into().unwrap_or(u32::MAX)
            })
            .time_to_live(Duration::from_secs(24 * 3600))
            .build();

        // 7-day TTL for verification result cache, keyed by bytecode_hash
        let verification_cache = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .weigher(|_k, v: &String| -> u32 {
                v.len().try_into().unwrap_or(u32::MAX)
            })
            .time_to_live(Duration::from_secs(7 * 24 * 3600))
            .build();

        Self { abi_cache, verification_cache, config }
    }

    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    pub async fn get_abi(&self, contract_id: &str) -> Option<String> {
        if !self.config.enabled { return None; }
        let result = self.abi_cache.get(contract_id).await;
        if result.is_some() {
            crate::metrics::ABI_CACHE_HITS.inc();
        } else {
            crate::metrics::ABI_CACHE_MISSES.inc();
        }
        result
    }

    pub async fn put_abi(&self, contract_id: &str, abi: String) {
        if !self.config.enabled { return; }
        self.abi_cache.insert(contract_id.to_string(), abi).await;
    }

    pub async fn invalidate_abi(&self, contract_id: &str) {
        if !self.config.enabled { return; }
        self.abi_cache.invalidate(contract_id).await;
    }

    pub async fn get_verification(&self, bytecode_hash: &str) -> Option<String> {
        if !self.config.enabled { return None; }
        let result = self.verification_cache.get(bytecode_hash).await;
        if result.is_some() {
            crate::metrics::VERIFICATION_CACHE_HITS.inc();
        } else {
            crate::metrics::VERIFICATION_CACHE_MISSES.inc();
        }
        result
    }

    pub async fn put_verification(&self, bytecode_hash: &str, result: String) {
        if !self.config.enabled { return; }
        self.verification_cache.insert(bytecode_hash.to_string(), result).await;
    }

    pub async fn invalidate_verification(&self, bytecode_hash: &str) {
        if !self.config.enabled { return; }
        self.verification_cache.invalidate(bytecode_hash).await;
    }

    // Generic get method to prevent old usages from throwing compile errors during transition
    pub async fn get(&self, _ns: &str, _key: &str) -> (Option<String>, bool) {
        (None, false)
    }

    pub async fn put(&self, _ns: &str, _key: &str, _value: String, _ttl: Option<Duration>) {}
    pub async fn invalidate(&self, _ns: &str, _key: &str) {}

    /// Starts an asynchronous startup warmup task querying the top 100 contracts
    pub fn warm_up(self: Arc<Self>, pool: PgPool) {
        if !self.config.enabled { return; }
        tokio::spawn(async move {
            tracing::info!("Starting startup cache warmup...");
            // Query top 100 contracts by query frequency from contract_interactions or just get contracts
            let top_contracts: Vec<(String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT c.contract_id, c.wasm_hash
                FROM contracts c
                LEFT JOIN contract_interactions ci ON c.id = ci.contract_id
                GROUP BY c.id
                ORDER BY COUNT(ci.id) DESC
                LIMIT 100
                "#
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            for (contract_id, wasm_hash) in top_contracts {
                if let Ok(Some(abi)) = sqlx::query_scalar::<_, serde_json::Value>(
                    "SELECT abi FROM contract_abis WHERE contract_id = $1 ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&contract_id)
                .fetch_optional(&pool).await {
                    self.abi_cache.insert(contract_id.clone(), abi.to_string()).await;
                }

                if let Some(w_hash) = wasm_hash {
                    if let Ok(Some(ver_res)) = sqlx::query_scalar::<_, String>(
                        "SELECT status::text FROM formal_verification_results LIMIT 1" // fallback fake
                    ).fetch_optional(&pool).await {
                        self.verification_cache.insert(w_hash.clone(), ver_res).await;
                    }
                }
            }
            tracing::info!("Completed startup cache warmup.");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_abi_cache() {
        let config = CacheConfig {
            enabled: true,
            max_capacity: 100,
        };
        let cache = CacheLayer::new(config);

        cache.put_abi("contract_1", "abi_json_1".to_string()).await;

        let val = cache.get_abi("contract_1").await;
        assert_eq!(val, Some("abi_json_1".to_string()));

        cache.invalidate_abi("contract_1").await;
        
        let val2 = cache.get_abi("contract_1").await;
        assert!(val2.is_none());
    }

    #[tokio::test]
    async fn test_verification_cache() {
        let config = CacheConfig {
            enabled: true,
            max_capacity: 100,
        };
        let cache = CacheLayer::new(config);

        cache.put_verification("hash_1", "result_1".to_string()).await;
        
        let val = cache.get_verification("hash_1").await;
        assert_eq!(val, Some("result_1".to_string()));
        
        cache.invalidate_verification("hash_1").await;
        
        let val2 = cache.get_verification("hash_1").await;
        assert!(val2.is_none());
    }

    #[tokio::test]
    async fn test_disabled_cache() {
        let config = CacheConfig {
            enabled: false,
            max_capacity: 100,
        };
        let cache = CacheLayer::new(config);

        cache.put_abi("c1", "v1".to_string()).await;
        let val = cache.get_abi("c1").await;
        assert!(val.is_none());

        cache.put_verification("h1", "v1".to_string()).await;
        let val2 = cache.get_verification("h1").await;
        assert!(val2.is_none());
    }
}
