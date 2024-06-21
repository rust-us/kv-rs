use std::collections::Bound;
use std::fmt::{Display, Formatter};
use std::ops::RangeBounds;
use crate::error::CResult;
use crate::storage::engine::Engine;
use crate::storage::{ScanIteratorT, Status};
use crate::storage::log::Log;

pub struct ManiFestCStore {

}

impl Display for ManiFestCStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ManiFestCStore")
    }
}

impl Engine for ManiFestCStore {
    type ScanIterator<'a> = CStoreLogScanIterator<'a>;

    fn delete(&mut self, key: &[u8]) -> CResult<i64> {
        todo!()
    }

    fn flush(&mut self) -> CResult<()> {
        todo!()
    }

    fn get(&mut self, key: &[u8]) -> CResult<Option<Vec<u8>>> {
        todo!()
    }

    fn scan(&mut self, range: impl RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_> where Self: Sized {
        todo!()
    }

    fn scan_dyn(&mut self, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> Box<dyn ScanIteratorT + '_> {
        Box::new(self.scan(range))
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> CResult<()> {
        todo!()
    }

    fn status(&mut self) -> CResult<Status> {
        todo!()
    }
}

impl Drop for ManiFestCStore {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            log::error!("failed to flush file: {}", error)
        }
    }
}

/// 用于进行范围读取
pub struct CStoreLogScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, (u64, u32)>,

    log: &'a mut Log,
}

impl<'a> CStoreLogScanIterator<'a> {
    /// map函数，调用self.log.read_value()去磁盘当中进行读取，用于将BTreeMap当中的key与offset转换为真实的kv。
    fn map(&mut self, item: (&Vec<u8>, &(u64, u32))) -> <Self as Iterator>::Item {
        let (key, (value_pos, value_len)) = item;

        Ok((key.clone(), self.log.read_value(*value_pos, *value_len)?))
    }
}

impl<'a> Iterator for CStoreLogScanIterator<'a> {
    type Item = CResult<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> DoubleEndedIterator for CStoreLogScanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|item| self.map(item))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
    }
}