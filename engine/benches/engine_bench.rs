use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use engine::engine::Engine;

fn criterion_benchmark(c: &mut Criterion) {
    let mut engine = Engine::new("main").unwrap();

    c.bench_function("get", |b| {
        b.iter(|| {
            let key = &"1".as_bytes().to_vec();
            let value = "2".as_bytes().to_vec();
            engine.set(black_box(key), black_box(value)).unwrap();
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
