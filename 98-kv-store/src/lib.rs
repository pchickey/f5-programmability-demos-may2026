//! The F5 key-value store API.

mod store;
pub use store::F5KvStore;

use anyhow::Result;
use std::pin::Pin;

/// A future trait object.
pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

/// An interface for key-value stores, which lets us plug in a filesystem-based
/// store during local development and the F5 store API for wasmtmm and nginx.
pub trait KvStore {
    /// Get the value associated with the given key, if any.
    fn get<'a>(&'a self, key: &'a [u8]) -> DynFuture<'a, Result<Option<Vec<u8>>>>;

    /// Associate `val` with the given key.
    fn set<'a>(&'a self, key: &'a [u8], val: &'a [u8]) -> DynFuture<'a, Result<()>>;

    /// Delete the value associated with the given key in this store.
    fn delete(&self, key: &[u8]) -> Result<()>;
}
