use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use crate::mvcc::Version;
use crate::storage::engine::Engine;

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