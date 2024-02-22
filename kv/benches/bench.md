# Bench

#### Add new content in the Cargo.toml file
```rust
[dev-dependencies]
criterion = "0.3"

[[bench]]
name = "my_benchmark"
harness = false

```


#### Create a test file in the project
> $PROJECT/benches/my_benchmark.rs

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

#### Run
@see [criterion.rs](https://bheisler.github.io/criterion.rs/book/getting_started.html)

```rust
cargo bench
```