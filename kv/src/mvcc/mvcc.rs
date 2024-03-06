//! This mod implements MVCC (Multi-Version Concurrency Control), a widely used method for ACID transactions and concurrency control.
//! It allows multiple concurrent transactions to access and modify the same dataset, isolates them from each other,
//! detects and handles conflicts, and commits their writes atomically as a single unit.
//! It uses an underlying storage engine to store raw keys and values.
//!
//!
//! VERSIONS
//! ========
//! MVCC handles concurrency control by managing multiple historical versions of keys, identified by a timestamp.
//! Every write adds a new version at a higher timestamp, with deletes having a special tombstone value.
//!
//! For example, the keys a,b,c,d may have the following values at various logical timestamps (x is tombstone):
//!
//! Time
//! 5
//! 4  a4
//! 3      b3      x
//! 2
//! 1  a1      c1  d1
//!    a   b   c   d   Keys
//!
//! * At time t1, a transaction writes a=a1,c=c1,d=d1 and commits it.
//! * At time t2, transaction T2 is started, will see the values a=a1, c=c1, d=d1.
//! * At t3, a transaction writes b=b3 and deletes D.
//! * At t4, a transaction writes a=a4.
//! * A different transaction t5 running at T=5 will see a=a4, b=b3, c=c1.
//!
//! KV Storage Engine uses logical timestamps with a sequence number stored in `Key::NextVersion`.
//! Each new read-write transaction takes its timestamp from the current value of `Key::NextVersion`
//! and then increments the value for the next transaction.
//!
//!
//! ISOLATION
//! =========
//! MVCC provides an isolation level called snapshot isolation.
//! Briefly, transactions see a consistent snapshot of the database state as of their start time.
//! Writes made by concurrent or subsequent transactions are never visible to it.
//! If two concurrent transactions write to the same key they will conflict and one of them must retry.
//! A transaction's writes become atomically visible to subsequent transactions only when they commit,
//! and are rolled back on failure.
//! Read-only transactions never conflict with other transactions.
//!
//! Transactions write new versions at their timestamp, storing them as `Key::Version(key, version) => value`.
//! If a transaction writes to a key and finds a newer version, it returns an error and the client must retry.
//!
//! Active (uncommitted) read-write transactions record their version in the active set,
//! stored as `Key::TxnActive(version)`.
//! When new transactions begin, they take a snapshot of this active set,
//! and any key versions that belong to a transaction in the active set are considered `invisible` (to anyone except that transaction itself).
//! Writes to keys that already have a past version in the active set will also return an error.
//!
//! To commit, a transaction simply deletes its record in the active set.
//! This will immediately (and, crucially, atomically) make all of its writes visible to subsequent transactions,
//! but not ongoing ones. If the transaction is cancelled and rolled back,
//! it maintains a record of all keys it wrote as `Key::TxnWrite(version, key)`,
//! so that it can find the corresponding versions and delete them before removing itself from the active set.
//!
//! For example, Consider the following example, where we have two ongoing transactions at time T=2 and T=5,
//! with some writes that are not yet committed marked in parentheses.
//!
//! Active set: [2, 5]
//!
//! Time
//! 5 (a5)
//! 4  a4
//! 3      b3      x
//! 2         (x)     (e2)
//! 1  a1      c1  d1
//!    a   b   c   d   e   Keys
//!
//! * (x): delete key
//! * (e2): put data but uncommit
//!
//! * The data written by transaction T5 is not committed, and T5 is maintained in the Active set.
//!   T5 does not see the tombstone at c@2 nor the value e=e2, because version=2 is in its active set.
//! * T2 deleting c1 and writing e2 are visible to itself, but not to the transaction T5 opened later.
//!   T2 will see a=a1, d=d1, e=e2 (it sees its own writes). T2 does not see any newer versions
//!
//! To commit, t2 can remove itself from the active set.
//! A new transaction t6 starting after the commit will then see c as deleted and e=e2.
//! t5 will still not see any of t2's writes, because it's still in its local snapshot of the active set at the time it began.
//!
//!
//! mvcc:
//!   Writers don't block readers.
//!   Readers don't block writers.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;
use crate::mvcc::transaction::{Transaction, TransactionDef, TransactionState};
use crate::mvcc::Version;
use crate::storage::engine::Engine;

/// An MVCC-based transactional key-value engine.
/// It wraps an underlying storage engine that's used for raw key/value storage.
pub struct MVCC<E: Engine> {
    engine: Arc<Mutex<E>>,
}

pub(crate) trait MVCCDef<E: Engine> {
    fn new(engine: E) -> MVCC<E>;

    fn begin(&self) -> CResult<Transaction<E>>;

    fn begin_read_only(&self) -> CResult<Transaction<E>>;

    fn resume(&self, state: TransactionState) -> CResult<Transaction<E>>;

    fn status(&self) -> CResult<Status>;
}

/// MVCC keys, using the KeyCode encoding which preserves the ordering and grouping of keys.
/// Cow byte slices allow encoding borrowed values and decoding into owned values.
#[derive(Debug, Deserialize, Serialize)]
pub enum Key<'a> {
    /// The next available version.
    NextVersion,

    /// Active (uncommitted) transactions by version.
    TxnActive(Version),

    /// `A snapshot of the active set` at each version.
    /// Only written for versions where the active set is non-empty (excluding itself).
    TxnActiveSnapshot(Version),

    /// Keeps track of all keys written to by an active transaction (identified by its version), in case it needs to roll back.
    TxnWrite(
        Version,
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),

    /// A versioned key/value pair.
    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
        Version,
    ),

    /// Unversioned non-transactional key/value pairs.
    /// These exist separately from versioned keys, i.e. the unversioned key "abcdefg" is entirely independent of the versioned key "abcdefg@7".
    /// These are mostly used for metadata.
    Unversioned(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),
}

impl<'a> Key<'a> {
    pub fn decode(bytes: &'a [u8]) -> CResult<Self> {
        todo!()
    }

    pub fn encode(&self) -> CResult<Vec<u8>> {
        todo!()
    }
}

/// MVCC key prefixes, for prefix scans. These must match the keys above, including the enum variant index.
#[derive(Debug, Deserialize, Serialize)]
enum KeyPrefix<'a> {
    NextVersion,

    TxnActive,

    TxnActiveSnapshot,

    TxnWrite(Version),

    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),

    Unversioned,
}

impl<'a> KeyPrefix<'a> {
    fn encode(&self) -> CResult<Vec<u8>> {
        todo!()
    }
}

impl <E: Engine> MVCCDef<E> for MVCC<E> {
    fn new(engine: E) -> MVCC<E> {
        MVCC {
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    fn begin(&self) -> CResult<Transaction<E>> {
        Transaction::begin(self.engine.clone())
    }

    fn begin_read_only(&self) -> CResult<Transaction<E>> {
        Transaction::begin_read_only(self.engine.clone(), None)
    }

    fn resume(&self, state: TransactionState) -> CResult<Transaction<E>> {
        Transaction::resume(self.engine.clone(), state)
    }

    fn status(&self) -> CResult<Status> {
        todo!()
    }
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