use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use engine::engine::Engine;

fn benchmark_set(c: &mut Criterion) {
    c.bench_function("set", |b| {
        b.iter_batched(
            || {
                let engine = Engine::new("bench_set").unwrap();
                let key = b"0001".to_vec();
                let value = b"value".to_vec();

                (engine, key, value)
            },
            |(mut engine, key, value)| {
                engine.set(black_box(&key), black_box(value)).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn benchmark_get_memtable(c: &mut Criterion) {
    let mut engine = Engine::new("bench_get_memtable").unwrap();

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();
        engine.set(&key, value).unwrap();
    }

    // "0999" was inserted last and remains in the active Memtable
    let key = b"0999".to_vec();

    c.bench_function("get_memtable", |b| {
        b.iter(|| {
            black_box(engine.get(black_box(&key)).unwrap());
        });
    });
}

fn benchmark_get_sstable(c: &mut Criterion) {
    let mut engine = Engine::new("bench_get_sstable").unwrap();

    // Force data into SSTables.
    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, value).unwrap();
    }

    let key = b"0500".to_vec();

    c.bench_function("get_sstable", |b| {
        b.iter(|| {
            black_box(engine.get(black_box(&key)).unwrap());
        });
    });
}

fn benchmark_get_missing(c: &mut Criterion) {
    let mut engine = Engine::new("bench_get_missing").unwrap();

    for i in 0..1000 {
        let key = format!("{:04}", i).into_bytes();
        let value = format!("value{:04}", i).into_bytes();

        engine.set(&key, value).unwrap();
    }

    let key = b"9999".to_vec();

    c.bench_function("get_missing", |b| {
        b.iter(|| {
            black_box(engine.get(black_box(&key)).unwrap());
        });
    });
}

fn benchmark_flush(c: &mut Criterion) {
    c.bench_function("set_with_flush", |b| {
        b.iter_batched(
            || Engine::new("bench_flush").unwrap(),
            |mut engine| {
                for i in 0..1000 {
                    let key = format!("{:04}", i).into_bytes();
                    let value = format!("value{:04}", i).into_bytes();

                    engine.set(&key, value).unwrap();
                }

                black_box(engine);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    benchmark_set,
    benchmark_get_memtable,
    benchmark_get_sstable,
    benchmark_get_missing,
    benchmark_flush
);

criterion_main!(benches);
