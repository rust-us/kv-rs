use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use crate::storage::engine::Engine;

/// mvcc
pub struct MVCC<E: Engine> {
    engine: Arc<Mutex<E>>,
}

impl<E: Engine> Clone for MVCC<E> {
    fn clone(&self) -> Self {
        MVCC { engine: self.engine.clone() }
    }
}

/// MVCC engine status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The total number of MVCC versions (i.e.  read-write transactions).
    pub versions: u64,
    /// Number of currently active transactions.
    pub active_txns: u64,
    /// The storage engine.
    pub storage: super::super::storage::Status,
}