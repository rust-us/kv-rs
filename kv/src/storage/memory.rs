use crate::error::CResult;
use crate::storage::engine::Engine;
use crate::storage::{ScanIteratorT, Status};

/// 纯内存的存储引擎，使用的就是BTreeMap，将key和value直接存储在内存当中，不会对数据进行持久化
pub struct Memory {
    data: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Memory {
    /// Creates Memory key-value storage engine.
    pub fn new() -> Self {
        Self { data: std::collections::BTreeMap::new() }
    }
}

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "memory")
    }
}

impl Engine for Memory {
    type ScanIterator<'a> = MemoryScanIterator<'a>;

    fn delete(&mut self, key: &[u8]) -> CResult<i64> {
        self.data.remove(key);

        Ok(1)
    }

    fn flush(&mut self) -> CResult<()> {
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> CResult<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }

    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_>
        where Self: Sized {
        MemoryScanIterator { inner: self.data.range(range) }
    }

    fn scan_dyn(
        &mut self,
        range: (std::ops::Bound<Vec<u8>>, std::ops::Bound<Vec<u8>>),
    ) -> Box<dyn ScanIteratorT + '_> {
        Box::new(self.scan(range))
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> CResult<()> {
        self.data.insert(key.to_vec(), value);
        Ok(())
    }

    fn status(&mut self) -> CResult<Status> {
        Ok(Status {
            name: self.to_string(),
            keys: self.data.len() as u64,
            size: self.data.iter().fold(0, |size, (k, v)| size + k.len() as u64 + v.len() as u64),
            total_disk_size: 0,
            live_disk_size: 0,
            garbage_disk_size: 0,
        })
    }
}

pub struct MemoryScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, Vec<u8>>,
}

impl<'a> MemoryScanIterator<'a> {
    fn map(item: (&Vec<u8>, &Vec<u8>)) -> <Self as Iterator>::Item {
        let (key, value) = item;
        Ok((key.clone(), value.clone()))
    }
}

impl<'a> Iterator for MemoryScanIterator<'a> {
    type Item = CResult<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Self::map)
    }
}

impl<'a> DoubleEndedIterator for MemoryScanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Self::map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    super::super::tests::test_engine!(Memory::new());
}
