pub mod log;
pub mod engine;
pub mod log_cask;
pub mod memory;
pub mod mani_fest_cstore;

use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;

/// KeyDir是一个内存当中的map，这里使用的是BTreeMap的实现方式，便于进行顺序遍历进行compaction。
/// key为存储的key，而value为Entry的metadata，记录长度和位置，用于进行偏移读取.
/// map当中始终保存当前key的最新版本的位置。 它便于顺序遍历和压缩。
pub type KeyDir = std::collections::BTreeMap<Vec<u8>, (u64, u32)>;

/// 用于表示当前存储引擎的状态
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The name of the storage engine.
    pub name: String,

    /// The number of live keys in the engine.
    pub keys: u64,

    /// The logical size of live key/value pairs.
    pub size: u64,

    /// The on-disk size of all data, live and garbage.
    pub total_disk_size: u64,

    /// The on-disk size of live data.
    pub live_disk_size: u64,
    
    /// The on-disk size of garbage data.
    pub garbage_disk_size: u64,
}

/// A scan iterator, with a blanket implementation (in lieu of trait aliases).
pub trait ScanIteratorT: DoubleEndedIterator<Item = CResult<(Vec<u8>, Vec<u8>)>> {}

impl<I: DoubleEndedIterator<Item = CResult<(Vec<u8>, Vec<u8>)>>> ScanIteratorT for I {}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        assert_eq!(1, 1);
    }

    /// Generates common tests for any Engine implementation.
    macro_rules! test_engine {
        ($setup:expr) => {
            #[track_caller]
            /// Asserts that a scan yields the expected items.
            fn assert_scan<I>(iter: I, expect: Vec<(&[u8], Vec<u8>)>) -> CResult<()>
            where
                I: Iterator<Item = CResult<(Vec<u8>, Vec<u8>)>>,
            {
                assert_eq!(
                    iter.collect::<CResult<Vec<_>>>()?,
                    expect.into_iter().map(|(k, v)| (k.to_vec(), v)).collect::<Vec<_>>()
                );
                Ok(())
            }

            /// Tests Engine point operations, i.e. set, get, and delete.
            #[test]
            fn point_ops() -> CResult<()> {
                let mut s = $setup;

                // Getting a missing key should return None.
                assert_eq!(s.get(b"a")?, None);

                // Setting and getting a key should return its value.
                s.set(b"a", vec![1])?;
                assert_eq!(s.get(b"a")?, Some(vec![1]));

                // Setting a different key should not affect the first.
                s.set(b"b", vec![2])?;
                assert_eq!(s.get(b"b")?, Some(vec![2]));
                assert_eq!(s.get(b"a")?, Some(vec![1]));

                // Getting a different missing key should return None. The
                // comparison is case-insensitive for strings.
                assert_eq!(s.get(b"c")?, None);
                assert_eq!(s.get(b"A")?, None);

                // Setting an existing key should replace its value.
                s.set(b"a", vec![0])?;
                assert_eq!(s.get(b"a")?, Some(vec![0]));

                // Deleting a key should remove it, but not affect others.
                s.delete(b"a")?;
                assert_eq!(s.get(b"a")?, None);
                assert_eq!(s.get(b"b")?, Some(vec![2]));

                // Deletes are idempotent.
                s.delete(b"a")?;
                assert_eq!(s.get(b"a")?, None);

                Ok(())
            }

            #[test]
            /// Tests Engine point operations on empty keys and values. These
            /// are as valid as any other key/value.
            fn point_ops_empty() -> CResult<()> {
                let mut s = $setup;
                assert_eq!(s.get(b"")?, None);
                s.set(b"", vec![])?;
                assert_eq!(s.get(b"")?, Some(vec![]));
                s.delete(b"")?;
                assert_eq!(s.get(b"")?, None);
                Ok(())
            }

            #[test]
            /// Tests Engine point operations on keys and values of increasing
            /// sizes, up to 16 MB.
            fn point_ops_sizes() -> CResult<()> {
                let mut s = $setup;

                // Generate keys/values for increasing powers of two.
                for size in (1..=24).map(|i| 1 << i) {
                    let bytes = "x".repeat(size);
                    let key = bytes.as_bytes();
                    let value = bytes.clone().into_bytes();

                    assert_eq!(s.get(key)?, None);
                    s.set(key, value.clone())?;
                    assert_eq!(s.get(key)?, Some(value));
                    s.delete(key)?;
                    assert_eq!(s.get(key)?, None);
                }

                Ok(())
            }

            #[test]
            /// Tests various Engine scans.
            fn scan() -> CResult<()> {
                let mut s = $setup;
                s.set(b"a", vec![1])?;
                s.set(b"b", vec![2])?;
                s.set(b"ba", vec![2, 1])?;
                s.set(b"bb", vec![2, 2])?;
                s.set(b"c", vec![3])?;
                s.set(b"C", vec![3])?;

                // Forward/reverse scans.
                assert_scan(
                    s.scan(b"b".to_vec()..b"bz".to_vec()),
                    vec![(b"b", vec![2]), (b"ba", vec![2, 1]), (b"bb", vec![2, 2])],
                )?;
                assert_scan(
                    s.scan(b"b".to_vec()..b"bz".to_vec()).rev(),
                    vec![(b"bb", vec![2, 2]), (b"ba", vec![2, 1]), (b"b", vec![2])],
                )?;

                // Inclusive/exclusive ranges.
                assert_scan(
                    s.scan(b"b".to_vec()..b"bb".to_vec()),
                    vec![(b"b", vec![2]), (b"ba", vec![2, 1])],
                )?;
                assert_scan(
                    s.scan(b"b".to_vec()..=b"bb".to_vec()),
                    vec![(b"b", vec![2]), (b"ba", vec![2, 1]), (b"bb", vec![2, 2])],
                )?;

                // Open ranges.
                assert_scan(s.scan(b"bb".to_vec()..), vec![(b"bb", vec![2, 2]), (b"c", vec![3])])?;
                assert_scan(
                    s.scan(..=b"b".to_vec()),
                    vec![(b"C", vec![3]), (b"a", vec![1]), (b"b", vec![2])],
                )?;

                // Full range.
                assert_scan(
                    s.scan(..),
                    vec![
                        (b"C", vec![3]),
                        (b"a", vec![1]),
                        (b"b", vec![2]),
                        (b"ba", vec![2, 1]),
                        (b"bb", vec![2, 2]),
                        (b"c", vec![3]),
                    ],
                )?;
                Ok(())
            }

            #[test]
            /// Tests prefix scans.
            fn scan_prefix() -> CResult<()> {
                let mut s = $setup;
                s.set(b"a", vec![1])?;
                s.set(b"b", vec![2])?;
                s.set(b"ba", vec![2, 1])?;
                s.set(b"bb", vec![2, 2])?;
                s.set(b"b\xff", vec![2, 0xff])?;
                s.set(b"b\xff\x00", vec![2, 0xff, 0x00])?;
                s.set(b"b\xffb", vec![2, 0xff, 2])?;
                s.set(b"b\xff\xff", vec![2, 0xff, 0xff])?;
                s.set(b"c", vec![3])?;
                s.set(b"\xff", vec![0xff])?;
                s.set(b"\xff\xff", vec![0xff, 0xff])?;
                s.set(b"\xff\xff\xff", vec![0xff, 0xff, 0xff])?;
                s.set(b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff])?;

                assert_scan(
                    s.scan_prefix(b""),
                    vec![
                        (b"a", vec![1]),
                        (b"b", vec![2]),
                        (b"ba", vec![2, 1]),
                        (b"bb", vec![2, 2]),
                        (b"b\xff", vec![2, 0xff]),
                        (b"b\xff\x00", vec![2, 0xff, 0x00]),
                        (b"b\xffb", vec![2, 0xff, 2]),
                        (b"b\xff\xff", vec![2, 0xff, 0xff]),
                        (b"c", vec![3]),
                        (b"\xff", vec![0xff]),
                        (b"\xff\xff", vec![0xff, 0xff]),
                        (b"\xff\xff\xff", vec![0xff, 0xff, 0xff]),
                        (b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(
                    s.scan_prefix(b"b"),
                    vec![
                        (b"b", vec![2]),
                        (b"ba", vec![2, 1]),
                        (b"bb", vec![2, 2]),
                        (b"b\xff", vec![2, 0xff]),
                        (b"b\xff\x00", vec![2, 0xff, 0x00]),
                        (b"b\xffb", vec![2, 0xff, 2]),
                        (b"b\xff\xff", vec![2, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(s.scan_prefix(b"bb"), vec![(b"bb", vec![2, 2])])?;

                assert_scan(s.scan_prefix(b"bq"), vec![])?;

                assert_scan(
                    s.scan_prefix(b"b\xff"),
                    vec![
                        (b"b\xff", vec![2, 0xff]),
                        (b"b\xff\x00", vec![2, 0xff, 0x00]),
                        (b"b\xffb", vec![2, 0xff, 2]),
                        (b"b\xff\xff", vec![2, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(
                    s.scan_prefix(b"b\xff\x00"),
                    vec![(b"b\xff\x00", vec![2, 0xff, 0x00])],
                )?;

                assert_scan(
                    s.scan_prefix(b"b\xff\xff"),
                    vec![(b"b\xff\xff", vec![2, 0xff, 0xff])],
                )?;

                assert_scan(
                    s.scan_prefix(b"\xff"),
                    vec![
                        (b"\xff", vec![0xff]),
                        (b"\xff\xff", vec![0xff, 0xff]),
                        (b"\xff\xff\xff", vec![0xff, 0xff, 0xff]),
                        (b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(
                    s.scan_prefix(b"\xff\xff"),
                    vec![
                        (b"\xff\xff", vec![0xff, 0xff]),
                        (b"\xff\xff\xff", vec![0xff, 0xff, 0xff]),
                        (b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(
                    s.scan_prefix(b"\xff\xff\xff"),
                    vec![
                        (b"\xff\xff\xff", vec![0xff, 0xff, 0xff]),
                        (b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff]),
                    ],
                )?;

                assert_scan(
                    s.scan_prefix(b"\xff\xff\xff\xff"),
                    vec![(b"\xff\xff\xff\xff", vec![0xff, 0xff, 0xff, 0xff])],
                )?;

                assert_scan(s.scan_prefix(b"\xff\xff\xff\xff\xff"), vec![])?;

                Ok(())
            }

            #[test]
            /// Runs random operations both on a Engine and a known-good
            /// BTreeMap, comparing the results of each operation as well as the
            /// final state.
            fn random_ops() -> CResult<()> {
                const NUM_OPS: u64 = 1000;

                use rand::{seq::SliceRandom, Rng, RngCore};
                let seed: u64 = rand::thread_rng().gen();
                let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
                println!("seed = {}", seed);

                #[derive(Debug)]
                enum Op {
                    Set,
                    Delete,
                    Get,
                    Scan,
                }

                impl rand::distributions::Distribution<Op> for rand::distributions::Standard {
                    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Op {
                        match rng.gen_range(0..=3) {
                            0 => Op::Set,
                            1 => Op::Delete,
                            2 => Op::Get,
                            3 => Op::Scan,
                            _ => panic!("unexpected value"),
                        }
                    }
                }

                let mut s = $setup;
                let mut keys: Vec<Vec<u8>> = Vec::new();
                let mut m = std::collections::BTreeMap::new();

                // Pick an already-used key with 80% probability, or generate a
                // new key.
                let mut random_key = |mut rng: &mut rand::rngs::StdRng| -> Vec<u8> {
                    if rng.gen::<f64>() < 0.8 && !keys.is_empty() {
                        keys.choose(&mut rng).unwrap().clone()
                    } else {
                        let mut key = vec![0; rng.gen_range(0..=16)];
                        rng.fill_bytes(&mut key);
                        keys.push(key.clone());
                        key
                    }
                };

                let random_value = |rng: &mut rand::rngs::StdRng| -> Vec<u8> {
                    let mut value = vec![0; rng.gen_range(0..=16)];
                    rng.fill_bytes(&mut value);
                    value
                };

                // Run random operations.
                for _ in 0..NUM_OPS {
                    match rng.gen::<Op>() {
                        Op::Set => {
                            let key = random_key(&mut rng);
                            let value = random_value(&mut rng);
                            println!("set {:?} = {:?}", key, value);
                            s.set(&key, value.clone())?;
                            m.insert(key, value);
                        }
                        Op::Delete => {
                            let key = random_key(&mut rng);
                            println!("delete {:?}", key);
                            s.delete(&key)?;
                            m.remove(&key);
                        }
                        Op::Get => {
                            let key = random_key(&mut rng);
                            let value = s.get(&key)?;
                            let expect = m.get(&key).cloned();
                            println!("get {:?} => {:?}", key, value);
                            assert_eq!(value, expect);
                        }
                        Op::Scan => {
                            let mut from = random_key(&mut rng);
                            let mut to = random_key(&mut rng);
                            if (to < from) {
                                (from, to) = (to, from)
                            }
                            println!("scan {:?} .. {:?}", from, to);
                            let result =
                                s.scan(from.clone()..to.clone()).collect::<CResult<Vec<_>>>()?;
                            let expect = m
                                .range(from..to)
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<Vec<_>>();
                            assert_eq!(result, expect);
                        }
                    }
                }

                // Compare the final states.
                println!("comparing final state");

                let state = s.scan(..).collect::<CResult<Vec<_>>>()?;
                let expect = m
                    .range::<Vec<u8>, _>(..)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>();
                assert_eq!(state, expect);

                Ok(())
            }

            #[test]
            /// Tests implementation-independent aspects of Status.
            fn status() -> CResult<()> {
                let mut s = $setup;
                s.set(b"foo", vec![1, 2, 3])?;
                s.set(b"bar", vec![1])?;
                s.delete(b"bar")?;
                s.set(b"baz", vec![1])?;
                s.set(b"baz", vec![2])?;
                s.set(b"baz", vec![3])?;
                s.delete(b"qux")?;

                let status = s.status()?;
                assert!(status.name.len() > 0);
                assert_eq!(status.keys, 2);
                assert_eq!(status.size, 10);

                Ok(())
            }
        };
    }

    pub(super) use test_engine; // export for use in submodules
}


