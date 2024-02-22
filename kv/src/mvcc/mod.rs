pub mod mvcc;

/// An MVCC version represents a logical timestamp. The latest version is incremented
/// when beginning each read-write transaction.
type Version = u64;
