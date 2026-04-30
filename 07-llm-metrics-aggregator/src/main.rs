//! Example of running the LLM token counter.
//!
//! ```shell-session
//! $ curl http://localhost:8000 -d 'tell me about the sun, in five words or less'
//! ```

use f5_kv_store::F5KvStore;

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use wstd::http::body::Body;
use wstd::http::{Client, Error, Request, Response};

const PROMPT_COUNT_KEY: &str = "total-prompt-eval-count";
const RESPONSE_COUNT_KEY: &str = "total-eval-count";

#[wstd::http_server]
async fn main(mut req: Request<Body>) -> Result<Response<Body>, Error> {
    let kv_store = F5KvStore::new().await?;
    let prompt = req.body_mut().str_contents().await?;

    let proxied_request = Request::post("http://127.0.0.1:11434/api/generate")
        .body(Body::from_json(&serde_json::json!({
            "model": "gemma3:1b",
            "stream": "false",
            "prompt": prompt,
        }))?)
        .expect("construct request POST /api/generate to olamma");

    let mut response = Client::new()
        .send(proxied_request)
        .await
        .context("olamma should be running at 127.0.0.1:11434 and respond to POST /api/generate")?;

    let mut val: Value = response
        .body_mut()
        .json()
        .await
        .context("/api/generate response body should be json")?;

    let count_sums = update_token_count_sums(&kv_store, &val)
        .await
        .with_context(|| format!("collecting token counts from {val:?}"))?;

    let object = val
        .as_object_mut()
        .ok_or_else(|| anyhow!("response body was object"))?;
    object.insert(
        PROMPT_COUNT_KEY.to_owned(),
        Value::from(count_sums.total_prompt_eval_count),
    );
    object.insert(
        RESPONSE_COUNT_KEY.to_owned(),
        Value::from(count_sums.total_eval_count),
    );

    *response.body_mut() = Body::from_json(&val)?;
    response.headers_mut().remove("content-length");
    Ok(response)
}

#[derive(Debug)]
pub struct TokenCounts {
    pub total_prompt_eval_count: u64,
    pub total_eval_count: u64,
}

async fn update_token_count_sums(kv_store: &F5KvStore, response: &Value) -> Result<TokenCounts> {
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

    let old_prompt_eval_count = kv_store
        .get(PROMPT_COUNT_KEY.as_bytes())
        .await?
        .map_or(0, |v| u64::from_le_bytes(v.as_slice().try_into().unwrap()));
    let old_eval_count = kv_store
        .get(RESPONSE_COUNT_KEY.as_bytes())
        .await?
        .map_or(0, |v| u64::from_le_bytes(v.as_slice().try_into().unwrap()));

    let new_prompt_eval_count = old_prompt_eval_count + prompt_eval_count;
    let new_eval_count = old_eval_count + eval_count;

    kv_store
        .set(
            PROMPT_COUNT_KEY.as_bytes(),
            new_prompt_eval_count.to_le_bytes().as_slice(),
        )
        .await?;
    kv_store
        .set(
            RESPONSE_COUNT_KEY.as_bytes(),
            new_eval_count.to_le_bytes().as_slice(),
        )
        .await?;

    Ok(TokenCounts {
        total_prompt_eval_count: new_prompt_eval_count,
        total_eval_count: new_eval_count,
    })
}
