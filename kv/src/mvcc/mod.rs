pub mod mvcc;
mod mvcc_test;
pub mod transaction;
mod transaction_test;
mod scan;

/// An MVCC version represents a logical timestamp. The latest version is incremented
/// when beginning each read-write transaction.
type Version = u64;
