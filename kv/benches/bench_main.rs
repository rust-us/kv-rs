use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

fn codec_bytes(num: u64) -> u64 {
    let list = MockStu::get_mock_list(num as usize);

    match num {
        0 => 1,
        1 => 1,
        n => codec_bytes(n-1) + codec_bytes(n-2),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MockStu {
    name: String,
    age: u32,
    address: String,
    sex: u8,
}

impl MockStu {
    fn get_mock_list(num: usize) -> Vec<MockStu> {
        let mut list = Vec::with_capacity(num);
        let mut rng = rand::thread_rng();

        for _ in 0..num {
            let s = MockStu {
                name: "张三".to_string(),
                age: rng.gen_range(0..80),
                address: "杭州余杭区".to_string(),
                sex: 1,
            };
            list.push(s);
        }

        list
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("codec bytes 20", |b| b.iter(|| codec_bytes(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);