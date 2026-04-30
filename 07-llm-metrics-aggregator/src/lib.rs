//! A proof-of-concept LLM token counter.

use anyhow::Result;
use f5_kv_store::F5KvStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wstd::http::Error;

/// The token counter.
pub struct LLMTokenCounter {
    kv_store: F5KvStore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCounts {
    pub total_prompt_eval_count: u64,
    pub total_eval_count: u64,
}

const PROMPT_COUNT_KEY: &[u8] = b"total-prompt-eval-count";
const RESPONSE_COUNT_KEY: &[u8] = b"total-eval-count";

impl LLMTokenCounter {
    /// Create a new LLM token counter with the given key-value store and
    /// configuration.
    pub fn new(kv_store: F5KvStore) -> Self {
        Self { kv_store }
    }

    /// Collect and proccess token metrics.
    pub async fn collect(&self, response: &Value) -> Result<TokenCounts> {
        let prompt_eval_count = response
            .get("prompt_eval_count")
            .ok_or(Error::msg("body json has prompt_eval_count"))?
            .as_u64()
            .ok_or(Error::msg("prompt_eval_count is not u64"))?;
        let eval_count = response
            .get("eval_count")
            .ok_or(Error::msg("body json has eval_count"))?
            .as_u64()
            .ok_or(Error::msg("eval_count is not u64"))?;

        let old_prompt_eval_count = self
            .kv_store
            .get(PROMPT_COUNT_KEY)
            .await?
            .map_or(0, |v| u64::from_le_bytes(v.as_slice().try_into().unwrap()));
        let old_eval_count = self
            .kv_store
            .get(RESPONSE_COUNT_KEY)
            .await?
            .map_or(0, |v| u64::from_le_bytes(v.as_slice().try_into().unwrap()));

        let new_prompt_eval_count = old_prompt_eval_count + prompt_eval_count;
        let new_eval_count = old_eval_count + eval_count;

        self.kv_store
            .set(
                PROMPT_COUNT_KEY,
                new_prompt_eval_count.to_le_bytes().as_slice(),
            )
            .await?;
        self.kv_store
            .set(RESPONSE_COUNT_KEY, new_eval_count.to_le_bytes().as_slice())
            .await?;

        Ok(TokenCounts {
            total_prompt_eval_count: new_prompt_eval_count,
            total_eval_count: new_eval_count,
        })
    }
}
