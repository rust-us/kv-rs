use std::collections::Bound;
use std::sync::MutexGuard;
use tokio_stream::StreamExt;
use crate::error::CResult;
use crate::mvcc::transaction::TransactionState;
use crate::mvcc::Version;
use crate::storage::engine::Engine;

/// A scan result. Can produce an iterator or collect an owned Vec.
///
/// This intermediate struct is unfortunately needed to hold the MutexGuard for the scan() caller,
/// since placing it in ScanIterator along with the inner iterator borrowing from it would create a self-referential struct.
pub struct Scan<'a, E: Engine + 'a> {
    /// Access to the locked engine.
    engine: MutexGuard<'a, E>,

    /// The transaction state.
    txn: &'a TransactionState,

    /// The scan type and parameter.
    param: ScanType,
}

enum ScanType {
    Range((Bound<Vec<u8>>, Bound<Vec<u8>>)),
    Prefix(Vec<u8>),
}

impl<'a, E: Engine + 'a> Scan<'a, E> {
    fn new(engine: MutexGuard<'a, E>, txn: &'a TransactionState, start: Bound<Vec<u8>>, end: Bound<Vec<u8>>, ) -> Self {
        Self { engine, txn, param: ScanType::Range((start, end)) }
    }

    /// Creates a new prefix scan.
    fn new_prefix(engine: MutexGuard<'a, E>, txn: &'a TransactionState, prefix: Vec<u8>) -> Self {
        Self { engine, txn, param: ScanType::Prefix(prefix) }
    }
}

/// An iterator over the latest live and visible key/value pairs at the txn
/// version.
pub struct ScanIterator<'a, E: Engine + 'a> {
    /// Decodes and filters visible MVCC versions from the inner engine iterator.
    inner: std::iter::Peekable<VersionIterator<'a, E>>,

    last: Option<Vec<u8>>,
}

impl<'a, E: Engine + 'a> ScanIterator<'a, E> {
    fn new(txn: &'a TransactionState, inner: E::ScanIterator<'a>) -> Self {
        todo!()
    }

    /// Fallible next(), emitting the next item, or None if exhausted.
    fn try_next(&mut self) -> CResult<Option<(Vec<u8>, Vec<u8>)>> {
        todo!()
    }

    /// Fallible next_back(), emitting the next item from the back, or None if exhausted.
    fn try_next_back(&mut self) -> CResult<Option<(Vec<u8>, Vec<u8>)>> {
        todo!()
    }
}

impl<'a, E: Engine> Iterator for ScanIterator<'a, E> {
    type Item = CResult<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'a, E: Engine> DoubleEndedIterator for ScanIterator<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.try_next_back().transpose()
    }
}

/// An iterator that decodes raw engine key/value pairs into MVCC key/value versions, and skips invisible versions. Helper for ScanIterator.
struct VersionIterator<'a, E: Engine + 'a> {
    /// The transaction the scan is running in.
    txn: &'a TransactionState,

    /// The inner engine scan iterator.
    inner: E::ScanIterator<'a>,
}

impl<'a, E: Engine> VersionIterator<'a, E> {
    fn new(txn: &'a TransactionState, inner: E::ScanIterator<'a>) -> Self {
        todo!()
    }

    /// Fallible next(), emitting the next item, or None if exhausted.
    fn try_next(&mut self) -> CResult<Option<(Vec<u8>, Version, Vec<u8>)>> {
        todo!()
    }

    /// Fallible next_back(), emitting the previous item, or None if exhausted.
    fn try_next_back(&mut self) -> CResult<Option<(Vec<u8>, Version, Vec<u8>)>> {
        todo!()
    }
}

impl<'a, E: Engine> Iterator for VersionIterator<'a, E> {
    type Item = CResult<(Vec<u8>, Version, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'a, E: Engine> DoubleEndedIterator for VersionIterator<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.try_next_back().transpose()
    }
}