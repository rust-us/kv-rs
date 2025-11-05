use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use kv_rs::encoding::{
    EncodingEngine, EncodingFormat, Base64Codec, HexCodec, JsonCodec, DataCodec, FormatDetector
};
use rand::Rng;

/// Performance benchmarks for encoding functionality
/// Tests encoding/decoding performance, format detection, and bulk operations

fn create_test_engine() -> EncodingEngine {
    let mut engine = EncodingEngine::new(EncodingFormat::Base64);
    engine.register_codec(EncodingFormat::Base64, Box::new(Base64Codec::new()));
    engine.register_codec(EncodingFormat::Hex, Box::new(HexCodec::new()));
    engine.register_codec(EncodingFormat::Json, Box::new(JsonCodec::new()));
    engine
}

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

fn generate_text_data(size: usize) -> Vec<u8> {
    let text = "The quick brown fox jumps over the lazy dog. ";
    let mut data = Vec::with_capacity(size);
    while data.len() < size {
        data.extend_from_slice(text.as_bytes());
    }
    data.truncate(size);
    data
}

fn bench_codec_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec_encoding");
    
    let sizes = vec![64, 256, 1024, 4096, 16384, 65536];
    
    for size in sizes {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark Base64 encoding
        group.bench_with_input(
            BenchmarkId::new("base64_encode", size),
            &data,
            |b, data| {
                let codec = Base64Codec::new();
                b.iter(|| codec.encode(black_box(data)).unwrap());
            },
        );
        
        // Benchmark Hex encoding
        group.bench_with_input(
            BenchmarkId::new("hex_encode", size),
            &data,
            |b, data| {
                let codec = HexCodec::new();
                b.iter(|| codec.encode(black_box(data)).unwrap());
            },
        );
        
        // Benchmark JSON encoding (for text data)
        let text_data = generate_text_data(size);
        group.bench_with_input(
            BenchmarkId::new("json_encode", size),
            &text_data,
            |b, data| {
                let codec = JsonCodec::new();
                b.iter(|| codec.encode(black_box(data)).unwrap());
            },
        );
    }
    
    group.finish();
}

fn bench_codec_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec_decoding");
    
    let sizes = vec![64, 256, 1024, 4096, 16384, 65536];
    
    for size in sizes {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));
        
        // Prepare encoded data
        let base64_codec = Base64Codec::new();
        let hex_codec = HexCodec::new();
        let json_codec = JsonCodec::new();
        
        let base64_encoded = base64_codec.encode(&data).unwrap();
        let hex_encoded = hex_codec.encode(&data).unwrap();
        
        let text_data = generate_text_data(size);
        let json_encoded = json_codec.encode(&text_data).unwrap();
        
        // Benchmark Base64 decoding
        group.bench_with_input(
            BenchmarkId::new("base64_decode", size),
            &base64_encoded,
            |b, encoded| {
                let codec = Base64Codec::new();
                b.iter(|| codec.decode(black_box(encoded)).unwrap());
            },
        );
        
        // Benchmark Hex decoding
        group.bench_with_input(
            BenchmarkId::new("hex_decode", size),
            &hex_encoded,
            |b, encoded| {
                let codec = HexCodec::new();
                b.iter(|| codec.decode(black_box(encoded)).unwrap());
            },
        );
        
        // Benchmark JSON decoding
        group.bench_with_input(
            BenchmarkId::new("json_decode", size),
            &json_encoded,
            |b, encoded| {
                let codec = JsonCodec::new();
                b.iter(|| codec.decode(black_box(encoded)).unwrap());
            },
        );
    }
    
    group.finish();
}

fn bench_format_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_detection");
    
    let base64_codec = Base64Codec::new();
    let hex_codec = HexCodec::new();
    let json_codec = JsonCodec::new();
    
    // Prepare test data of different sizes
    let sizes = vec![64, 256, 1024, 4096];
    
    for size in sizes {
        let data = generate_test_data(size);
        let text_data = generate_text_data(size);
        
        let base64_encoded = base64_codec.encode(&data).unwrap();
        let hex_encoded = hex_codec.encode(&data).unwrap();
        let json_encoded = json_codec.encode(&text_data).unwrap();
        
        let detector = FormatDetector::new();
        
        // Benchmark Base64 detection
        group.bench_with_input(
            BenchmarkId::new("detect_base64", size),
            &base64_encoded,
            |b, encoded| {
                b.iter(|| detector.detect(black_box(encoded)));
            },
        );
        
        // Benchmark Hex detection
        group.bench_with_input(
            BenchmarkId::new("detect_hex", size),
            &hex_encoded,
            |b, encoded| {
                b.iter(|| detector.detect(black_box(encoded)));
            },
        );
        
        // Benchmark JSON detection
        group.bench_with_input(
            BenchmarkId::new("detect_json", size),
            &json_encoded,
            |b, encoded| {
                b.iter(|| detector.detect(black_box(encoded)));
            },
        );
        
        // Benchmark mixed format detection (ambiguous data)
        let ambiguous_data = "41414141"; // Could be hex or base64
        group.bench_function(
            &format!("detect_ambiguous_{}", size),
            |b| {
                b.iter(|| detector.detect(black_box(ambiguous_data)));
            },
        );
    }
    
    group.finish();
}

fn bench_encoding_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_engine");
    
    let mut engine = create_test_engine();
    let sizes = vec![64, 256, 1024, 4096];
    
    for size in sizes {
        let data = generate_test_data(size);
        group.throughput(Throughput::Bytes(size as u64));
        
        // Benchmark engine encoding operations
        group.bench_with_input(
            BenchmarkId::new("engine_encode_base64", size),
            &data,
            |b, data| {
                b.iter(|| engine.encode(black_box(data), EncodingFormat::Base64).unwrap());
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("engine_encode_hex", size),
            &data,
            |b, data| {
                b.iter(|| engine.encode(black_box(data), EncodingFormat::Hex).unwrap());
            },
        );
        
        // Benchmark engine decoding operations
        let base64_encoded = engine.encode(&data, EncodingFormat::Base64).unwrap();
        let hex_encoded = engine.encode(&data, EncodingFormat::Hex).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("engine_decode_base64", size),
            &base64_encoded,
            |b, encoded| {
                b.iter(|| engine.decode(black_box(encoded), EncodingFormat::Base64).unwrap());
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("engine_decode_hex", size),
            &hex_encoded,
            |b, encoded| {
                b.iter(|| engine.decode(black_box(encoded), EncodingFormat::Hex).unwrap());
            },
        );
        
        // Benchmark auto-detection with decoding
        group.bench_with_input(
            BenchmarkId::new("engine_detect_and_decode", size),
            &base64_encoded,
            |b, encoded| {
                b.iter(|| {
                    let detected = engine.detect(black_box(encoded)).unwrap();
                    if !detected.is_empty() {
                        engine.decode(encoded, detected[0].format).unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");
    
    let engine = create_test_engine();
    let batch_sizes = vec![10, 50, 100, 500];
    
    for batch_size in batch_sizes {
        // Prepare batch data
        let mut batch_data = Vec::new();
        for i in 0..batch_size {
            let data = format!("test_data_{}", i).into_bytes();
            batch_data.push(data);
        }
        
        group.throughput(Throughput::Elements(batch_size as u64));
        
        // Benchmark bulk encoding
        group.bench_with_input(
            BenchmarkId::new("bulk_encode_base64", batch_size),
            &batch_data,
            |b, data| {
                b.iter(|| {
                    for item in data {
                        engine.encode(black_box(item), EncodingFormat::Base64).unwrap();
                    }
                });
            },
        );
        
        // Benchmark bulk decoding
        let encoded_batch: Vec<String> = batch_data
            .iter()
            .map(|data| engine.encode(data, EncodingFormat::Base64).unwrap())
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("bulk_decode_base64", batch_size),
            &encoded_batch,
            |b, encoded_data| {
                b.iter(|| {
                    for encoded in encoded_data {
                        engine.decode(black_box(encoded), EncodingFormat::Base64).unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    let mut engine = EncodingEngine::with_cache_settings(
        EncodingFormat::Base64,
        std::time::Duration::from_secs(300),
        1000,
    );
    engine.register_codec(EncodingFormat::Base64, Box::new(Base64Codec::new()));
    engine.register_codec(EncodingFormat::Hex, Box::new(HexCodec::new()));
    
    let test_data = vec![
        "SGVsbG8gV29ybGQ=",  // Base64
        "48656c6c6f20576f726c64",  // Hex
        "VGVzdCBkYXRh",  // Base64
        "54657374",  // Hex
    ];
    
    // Benchmark cache hit performance
    group.bench_function("cache_hit_detection", |b| {
        // Pre-populate cache
        for data in &test_data {
            engine.detect(data).unwrap();
        }
        
        b.iter(|| {
            for data in &test_data {
                engine.detect(black_box(data)).unwrap();
            }
        });
    });
    
    // Benchmark cache miss performance
    group.bench_function("cache_miss_detection", |b| {
        engine.clear_cache();
        
        b.iter(|| {
            engine.clear_cache();
            for data in &test_data {
                engine.detect(black_box(data)).unwrap();
            }
        });
    });
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    let engine = create_test_engine();
    let sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB
    
    for size in sizes {
        let data = generate_test_data(size);
        
        // Benchmark memory allocation during encoding
        group.bench_with_input(
            BenchmarkId::new("memory_encode_base64", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let encoded = engine.encode(black_box(data), EncodingFormat::Base64).unwrap();
                    black_box(encoded);
                });
            },
        );
        
        // Benchmark memory allocation during decoding
        let encoded = engine.encode(&data, EncodingFormat::Base64).unwrap();
        group.bench_with_input(
            BenchmarkId::new("memory_decode_base64", size),
            &encoded,
            |b, encoded| {
                b.iter(|| {
                    let decoded = engine.decode(black_box(encoded), EncodingFormat::Base64).unwrap();
                    black_box(decoded);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    encoding_benches,
    bench_codec_encoding,
    bench_codec_decoding,
    bench_format_detection,
    bench_encoding_engine,
    bench_bulk_operations,
    bench_cache_performance,
    bench_memory_usage
);

criterion_main!(encoding_benches);