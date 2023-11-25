use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oid::{generate_oid, Oid};

fn benchmark(c: &mut Criterion) {
    let mut oid = Oid::new();

    c.bench_function("generate_oid() (global)", |b| {
        b.iter(|| black_box(generate_oid()));
    });

    c.bench_function("oid.generate()", |b| {
        b.iter(|| black_box(oid.generate()));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
