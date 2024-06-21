
#[cfg(test)]
mod tx_test {
    use std::sync::{Arc, Mutex};
    use crate::error::CResult;
    use crate::mvcc::transaction::{Transaction, TransactionDef};
    use crate::storage::memory::Memory;

    #[test]
    fn tx_test() -> CResult<()> {
        let engine = Memory::new();

        let tx = Transaction::begin(Arc::new(Mutex::new(engine)))?;
        assert_eq!(tx.version(), 1);

        Ok(())
    }
}