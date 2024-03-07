//! 这个模块实现了MVCC（多版本并发控制），这是一种广泛使用的ACID事务和并发控制方法。
//! 它允许多个并发事务访问和修改同一数据集，将它们彼此隔离，
//! 检测和处理冲突，并将其写入作为一个单元进行原子化提交。
//! 它使用底层存储引擎来存储原始键和值。
//!
//! VERSIONS
//! ========
//! MVCC通过管理由时间戳标识的密钥的多个历史版本来处理并发控制。
//! 每次写入都会以更高的时间戳添加一个新版本，删除时会有一个特殊的tombstone值。
//!
//! For example, the keys a,b,c,d 可以在任何 logical timestamps 的维度下有如下值的变化(x 逻辑删除):
//!
//! <code>
//! Time
//! 5
//! 4  a4
//! 3      b3      x
//! 2
//! 1  a1      c1  d1
//!    a   b   c   d   Keys
//! </code>
//!
//! * 在t1时刻，事务写入a=a1，c=c1，d=d1并提交它。
//! * 在t2时刻，开启了事务t2，将看到值a＝a1，c＝c1，d＝d1。
//! * 在t3时刻，事务写入b＝b3, 并删除d。
//! * 在t4时刻，事务写入a=a4。
//! * 在T＝5处运行的不同事务t5将看到A＝a4，b＝b3，c＝c1。
//!
//!
//! KV存储引擎使用逻辑时间戳和存储在 `Key::NextVersion` 中的序列号。
//! 每个新的读写事务都从 `Key::NextVersion` 的当前值中获取其时间戳，然后增加下一个事务的值。
//!
//!
//! ISOLATION
//! =========
//! MVCC提供了一种称为快照隔离的隔离级别。
//! 简而言之，事务在开始时看到数据库状态的一致快照。并发或后续事务所做的写入对它来说永远是不可见的。
//! 如果两个并发事务写入相同的键，它们会发生冲突，其中一个必须重试。
//! 事务的写入只有在提交时才能被后续事务原子地看到，并在失败时回滚。
//! 只读事务永远不会与其他事务冲突。
//!
//! 事务会在时间戳处写入新版本，并将它们存储为 `Key::Version(key, version) => value`。
//! 如果事务写入一个键并发现一个较新的版本，它将返回一个错误，客户端必须重试。
//!
//! 活动（未提交）读写事务在活动集中记录它们的版本，存储为  `Key::TxnActive(version)`。
//! 当新事务开始时，它们会获取这个活动集的快照， 并且属于活动集中事务的任何密钥版本都被认为是“不可见的”（除了该事务本身之外的任何人）。
//! 写入已包含活动集中的旧版本的密钥也将返回错误。
//!
//! 提交事务时，只需删除活动集中的记录。这将立即（并且至关重要地）使所有后续事务的写入可见，但不会对正在进行的事务可见。
//! 如果事务被取消和回滚，它将保留所有写入的键的记录，如 `Key::TxnWrite(version, key)`，以便它可以在从活动集中删除自身之前找到对应的版本并将其删除。
//!
//! 记录真正变成可见是根据提交的时刻决定的，在事务未提交前，该事务写入的数据对于自己是可见的，但是对于其他的事务不可见。
//!
//! For example, 考虑以下示例，其中我们在时间T=2和T=5有两个正在进行的交易，括号中标记了一些尚未提交的写入。.
//!
//! Active set: [2, 5]
//!
//! <code>
//! Time
//! 5 (a5)
//! 4  a4
//! 3      b3      x
//! 2         (x)     (e2)
//! 1  a1      c1  d1
//!    a   b   c   d   e   Keys
//! </code>
//!
//! * (x): delete key
//! * (e2): put data but uncommit
//!
//! * 事务T5写入的数据未提交，T5保持在活动集中。 T5 看不到 c@2 处的墓碑删除动作，也看不到值 e=e2，因为版本=2 处于其活动集中，未提交。
//! * T2删除c1并写入e2的操作对它自己可见，但对稍后打开的事务T5不可见。T2 将看到 a=a1、d=d1、e=e2（它看到的是它自己的写入）。T2 看不到任何更新版本。
//!
//! 如果t2存在事物提交， t2将会从事物活动集中被删除。 在t2提交后开始的新事务 t6 将看到 c 被删除，并且 e=e2。
//! t5 仍然不会看到 t2 的任何写入，因为它仍然处于开始时的活动集的本地快照中。
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

/// 基于MVCC的事务键值引擎，提供最基本的ACID和MVCC支持。它包装了一个用于键/值存储的基础存储引擎。
/// MVCC所提供的隔离级别为快照隔离，事务只能看到数据库的一个一致性快照，而这个快照是根据事务创建的时间决定的，即事务只能够看到事务创建前的最新的数据，以及由自己写入的新数据。
/// 目前还未提交的活跃事务之间相互隔离互不影响。
///
/// 会保存数据的所有版本，因此就可以支持time travel query，即传入一个时间戳，然后获取一个那时的快照，
/// 进行只读请求(由于基于旧版本进行写请求会扰乱当前数据库的状态，如进行set x = x + 1，原本x = 3，但是目前已经是x = 5了，因此time travel只支持只读事务)
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

/// MVCC键，使用KeyCode编码，保留键的顺序和分组。
/// Cow byte slices允许对借用值进行编码并将其解码为自有值。
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

    /// 无版本的非事务性键/值对。
    /// 这些与versioned keys分开存在，即未版本化的 key  “abcdefg” 完全独立于 versioned key “abcdefg@7”。 这些主要用于元数据。
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

/// MVCC 键前缀，用于前缀扫描。这些必须与上面的键匹配，包括枚举变量索引。
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

/// 表示当前事务的状态
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The total number of MVCC versions (i.e.  read-write transactions).
    pub versions: u64,

    /// Number of currently active transactions.
    pub active_txns: u64,

    /// The storage engine.
    /// storage是存储引擎的storage
    pub storage: super::super::storage::Status,
}