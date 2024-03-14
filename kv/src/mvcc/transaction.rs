use std::collections::{Bound, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex, MutexGuard};
use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;
use crate::mvcc::mvcc::Key;
use crate::mvcc::scan::Scan;
use crate::mvcc::transaction::seals::EngineSealedMut;
use crate::mvcc::Version;
use crate::storage::engine::Engine;
use crate::storage::{ScanIteratorT, Status};

/// 事务最基础的结构体
pub struct Transaction<E: Engine> {
    /// The underlying engine, shared by all transactions.
    engine: Arc<Mutex<E>>,

    /// The transaction state.
    st: TransactionState,
}

/// 表示事务的状态。 事务状态的设计使得事务可以在不同的组件之间安全地传递，并且可以在不直接引用事务本身的情况下被引用，有助于简化事务管理。
///
/// 事务的状态，它决定了事务的写版本和隔离级别。 它与事务分开，允许它独立于引擎传递。
///
/// 这样做有两个主要动机：
///
/// - 它可以通过 `Transaction.state()` 导出，(de)serialized(（反）序列化)，然后通过 Transaction::resume() 实例化一个新的功能等价的 Transaction。
///   这允许跨机器边界在存储引擎之间传递事务（以支持未来的分布式功能）。
///
/// - 它可以独立于 Engine 借用，允许在 VisibleIterator 中引用它，否则会导致自引用。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionState {
    /// 此事务正在运行的版本。 在给定的版本中只能运行一个读写事务，因为这会标识其写入。
    pub version: Version,

    /// If true, the transaction is read only.
    pub read_only: bool,

    /// 截至此事务开始时，并发活动（未提交）事务的集合。 即使他们正在写入较低版本，他们的写入也应对于此事务不可见，因为他们尚未提交。
    pub active: HashSet<Version>,
}

pub trait TransactionStateDef {
    /// 判断给定的version对于当前事务是否可见。
    ///
    /// 未来版本以及从本次交易开始时属于有效交易的版本永远不可见。
    ///
    /// 读写事务在它们的版本中看到自己的写入。
    ///
    /// 只读查询只能看到事务版本以下的版本，不包括版本本身。
    /// 这是为了确保 time-travel 查询在该版本提交写入之前和之后都能看到一致的版本。
    fn is_visible(&self, version: Version) -> bool;
}

impl TransactionStateDef for TransactionState {
    fn is_visible(&self, version: Version) -> bool {
        if self.active.get(&version).is_some() { // 如果version来自活跃事务，即处于active_set当中，那么代表为新写入的，不可见
            false
        } else if self.read_only { // 如果为只读事务，那么能看到小于version的(之前事务创建的)
            version < self.version
        } else { // 如果是普通事务，那么可以看到之前的和自身写入(<=)
            version <= self.version
        }
    }
}

pub trait TransactionDef<E: Engine> {
    /// 开启一个read-write的新事务。 这将分配一个新版本，交易可以在其中写入， 将其添加到活动集，并记录其活动快照以进行时间旅行查询。
    fn begin(engine: Arc<Mutex<E>>) -> CResult<Transaction<E>>;

    /// Begins a new read-only transaction.
    /// 如果给出了版本，它将看到该版本开始时的状态（忽略该版本中的写入）。
    /// 换句话说，它看到的状态与该版本开始时的读写事务看到的状态相同。
    fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> CResult<Transaction<E>>;

    /// Resumes a transaction from the given state.
    fn resume(engine: Arc<Mutex<E>>, s: TransactionState) -> CResult<Self> where Self: Sized;

    /// 获取当前活动事务的集合。
    fn scan_active(session: &mut MutexGuard<E>) -> CResult<HashSet<Version>>;

    /// Returns the version the transaction is running at.
    fn version(&self) -> Version;

    /// Returns whether the transaction is read-only.
    fn is_read_only(&self) -> bool;

    /// Returns the transaction's state. This can be used to instantiate a functionally equivalent transaction via resume().
    fn state(&self) -> &TransactionState;

    /// 通过将其从活动集中删除，提交事务。 这将立即使后续事务的写入可见。 同时删除不再需要的 TxnWrite 记录。
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
        let mut session = engine.lock()?;

        // Allocate a new version 作为当前事务的tid,或者可以视为一个时间戳，之后+1写回。
        // 传统MYSQL的方式实现为 buffer pool + wal机制，此处采用的是在存储引擎当中同步持久化 NextVersion 的方式。
        let version = match session.get(&Key::NextVersion.encode()?)? {
            Some(ref v) => bincode::deserialize(v)?,
            None => 1,
        };
        session.set(&Key::NextVersion.encode()?, bincode::serialize(&(version + 1))?)?;

        // 从存储引擎当中扫描，恢复出当前的active_set。开启一个事务后，就向存储引擎当中写入一条Key::TxnActive，带上自己的version，之后扫描出所有Key::TxnActive的key，恢复出active_set，
        // 由于存储引擎本身是一个append-only的存储设计， 就算是将value设置为完整的active_set，那么每次写入也是追加写入，并且需要完整的写入整个active_set，写入量反而增大，
        // active_set只会在事务begin的时候进行读取
        let mut active = HashSet::new();

        Ok(
            Self {
                engine: engine.clone(),
                st: TransactionState {
                    version,
                    read_only: true,
                    active
                }
            }
        )
    }

    fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> CResult<Transaction<E>> {
        todo!()
    }

    fn resume(engine: Arc<Mutex<E>>, s: TransactionState) -> CResult<Self> where Self: Sized {
        todo!()
    }

    fn scan_active(session: &mut MutexGuard<E>) -> CResult<HashSet<Version>> {
        todo!()
    }

    fn version(&self) -> Version {
        self.st.version
    }

    fn is_read_only(&self) -> bool {
        self.st.read_only
    }

    fn state(&self) -> &TransactionState {
        &self.st
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