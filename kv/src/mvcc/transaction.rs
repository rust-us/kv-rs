use std::collections::{Bound, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex, MutexGuard};
use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;
use crate::mvcc::scan::Scan;
use crate::mvcc::transaction::seals::EngineSealedMut;
use crate::mvcc::Version;
use crate::storage::engine::Engine;
use crate::storage::{ScanIteratorT, Status};

/// An MVCC transaction.
pub struct Transaction<E: Engine> {
    /// The underlying engine, shared by all transactions.
    engine: Arc<Mutex<E>>,

    /// The transaction state.
    st: TransactionState,
}

/// A Transaction's state, which determines its write version and isolation.
/// It is separate from Transaction to allow it to be passed around independently of the engine.
/// There are two main motivations for this:
///
/// - It can be exported via Transaction.state(), (de)serialized,
///   and later used to instantiate a new functionally equivalent Transaction via Transaction::resume().
///   This allows passing the transaction between the storage engine across the machine boundary (To support distributed capabilities in the future.).
///
/// - It can be borrowed independently of Engine, allowing references to it in VisibleIterator,
///   which would otherwise result in self-references.
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

pub trait TransactionStateDef {
    /// Checks whether the given version is visible to this transaction.
    ///
    /// Future versions, and versions belonging to active transactions as of the start of this transaction, are never isible.
    ///
    /// Read-write transactions see their own writes at their version.
    ///
    /// Read-only queries only see versions below the transaction's version, excluding the version itself.
    /// This is to ensure time-travel queries see a consistent version both before and after any active transaction at that version commits its writes.
    fn is_visible(&self, version: Version) -> bool;
}

impl TransactionStateDef for TransactionState {
    fn is_visible(&self, version: Version) -> bool {
        if self.active.get(&version).is_some() {
            false
        } else if self.read_only {
            version < self.version
        } else {
            version <= self.version
        }
    }
}

pub trait TransactionDef<E: Engine> {
    /// Begins a new transaction in ** read-write mode **.
    /// This will allocate a new version that the transaction can write at,
    /// add it to the active set, and record its ** active snapshot for time-travel ** queries.
    fn begin(engine: Arc<Mutex<E>>) -> CResult<Transaction<E>>;

    /// Begins a new read-only transaction.
    /// If version is given it will see the state as of the beginning of that version (ignoring writes at that version).
    /// In other words, it sees the same state as the read-write transaction at that version saw when it began.
    fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> CResult<Transaction<E>>;

    /// Fetches the set of currently active transactions.
    fn scan_active(session: &mut MutexGuard<E>) -> CResult<HashSet<Version>>;

    /// Returns the version the transaction is running at.
    fn version(&self) -> Version;

    /// Returns whether the transaction is read-only.
    fn is_read_only(&self) -> bool;

    /// Returns the transaction's state. This can be used to instantiate a functionally equivalent transaction via resume().
    fn state(&self) -> &TransactionState;

    /// Commits the transaction, by ** removing it from the active set ** .
    /// This will immediately make its writes visible to subsequent transactions.
    /// Also removes its TxnWrite records, which are no longer needed.
    fn commit(self) -> CResult<()>;

    /// Rolls back the transaction, by undoing all written versions and removing it from the active set.
    /// The active set snapshot is left behind, since this is needed for time travel queries at this version.
    fn rollback(self) -> CResult<()>;

    /// Deletes a key.
    fn delete(&self, key: &[u8]) -> CResult<i64>;

    /// Sets a value for a key.
    fn set(&self, key: &[u8], value: Vec<u8>) -> CResult<()>;

    /// Fetches a key's value, or None if it does not exist.
    fn get(&self, key: &[u8]) -> CResult<Option<Vec<u8>>>;

    fn scan<R: RangeBounds<Vec<u8>>>(&self, range: R) -> CResult<Scan<E>>;
}

mod seals {
    use std::sync::{Arc, Mutex};
    use crate::storage::engine::Engine;

    pub trait EngineSealedMut<E: Engine> {
        fn get_engine(&self) -> Arc<Mutex<E>>;
    }
}

impl <E: Engine> EngineSealedMut<E> for Transaction<E> {
    fn get_engine(&self) -> Arc<Mutex<E>> {
        self.engine.clone()
    }
}

impl <E: Engine> Transaction<E> {
    /// write data with tx
    fn write_data() -> CResult<()> {
        todo!()
    }
}

impl <E: Engine> TransactionDef<E> for Transaction<E> {
    fn begin(engine: Arc<Mutex<E>>) -> CResult<Transaction<E>> {
        todo!()
    }

    fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> CResult<Transaction<E>> {
        todo!()
    }

    fn scan_active(session: &mut MutexGuard<E>) -> CResult<HashSet<Version>> {
        todo!()
    }

    fn version(&self) -> Version {
        todo!()
    }

    fn is_read_only(&self) -> bool {
        todo!()
    }

    fn state(&self) -> &TransactionState {
        todo!()
    }

    fn commit(self) -> CResult<()> {
        todo!()
    }

    fn rollback(self) -> CResult<()> {
        todo!()
    }

    fn delete(&self, key: &[u8]) -> CResult<i64> {
        todo!()
    }

    fn set(&self, key: &[u8], value: Vec<u8>) -> CResult<()> {
        todo!()
    }

    fn get(&self, key: &[u8]) -> CResult<Option<Vec<u8>>> {
        todo!()
    }

    fn scan<R: RangeBounds<Vec<u8>>>(&self, range: R) -> CResult<Scan<E>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
    }
}