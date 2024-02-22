use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use crate::mvcc::Version;
use crate::storage::engine::Engine;

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
    /// The total number of MVCC versions (i.e. read-write transactions).
    pub versions: u64,
    /// Number of currently active transactions.
    pub active_txns: u64,
    /// The storage engine.
    pub storage: super::super::storage::Status,
}

/// An MVCC transaction.
pub struct Transaction<E: Engine> {
    /// The underlying engine, shared by all transactions.
    engine: Arc<Mutex<E>>,

    /// The transaction state.
    st: TransactionState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionState {
    /// The version this transaction is running at.
    /// Only one read-write transaction can run at a given version, since this identifies its writes.
    pub version: Version,

    /// If true, the transaction is read only.
    pub read_only: bool,

    /// The set of concurrent active (uncommitted) transactions, as of the start of this transaction.
    /// Their writes should be invisible to this transaction even if they're writing at a lower version,
    /// since they're not committed yet.
    pub active: HashSet<Version>,
}