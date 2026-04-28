use super::*;
use anyhow::anyhow;

wit_bindgen::generate!({
    world: "my-world",
    path: [
        "wit/deps/io/wasi_io@0.2.3.wit",
        "wit/deps/f5/key-value.wit",
    ],
    inline: "
            package metrics:aggregator;

            world my-world {
                import f5:key-value/key-value-store@0.1.0;
            }
        ",
    with: {
        "wasi:io/poll@0.2.3": wasi::io::poll,
        "f5:key-value/key-value-store@0.1.0": generate,
    }
});

use self::f5::key_value::key_value_store::Store;

/// A key-value store backed by the F5 key-value store WIT implemented for
/// both wsmtmm and nginx.
#[non_exhaustive]
pub struct F5KvStore {
    store: Store,
}

const F5_STORE_NAME: &str = "f5";

async fn wait_for(pollable: wasi::io::poll::Pollable) {
    wstd::runtime::AsyncPollable::new(pollable).wait_for().await
}

impl F5KvStore {
    pub async fn new() -> Result<Self> {
        let store = Store::open(F5_STORE_NAME);
        wait_for(store.subscribe()).await;
        let store = store
            .get()
            .expect("first time calling `get`")
            .expect("`get` result should be `Some` because we blocked on the pollable's readiness")
            .map_err(|e| anyhow!("failed to open store: {e}"))?;
        Ok(Self { store })
    }
}

impl KvStore for F5KvStore {
    fn get<'a>(&'a self, key: &'a [u8]) -> DynFuture<'a, Result<Option<Vec<u8>>>> {
        Box::pin(async {
            let value = self.store.get(key);
            wait_for(value.subscribe()).await;
            let value = value
                .get()
                .expect("first time calling `get`")
                .expect("`get` should be `Some` because we blocked on the pollable's readiness")
                .map_err(|e| anyhow!("failed to get key: {e}"))?;
            Ok(value)
        })
    }

    fn set<'a>(&'a self, key: &'a [u8], val: &'a [u8]) -> DynFuture<'a, Result<()>> {
        Box::pin(async {
            self.store
                .set(key, val)
                .map_err(|e| anyhow!("failed to set key: {e}"))?;
            Ok(())
        })
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.store
            .delete(key)
            .map_err(|e| anyhow!("failed to delete key: {e}"))
    }
}
