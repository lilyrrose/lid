use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lid::{generate_lid, LID};

fn benchmark(c: &mut Criterion) {
    let mut lid = LID::new();

    c.bench_function("generate_lid() (global)", |b| {
        b.iter(|| black_box(generate_lid()));
    });

    c.bench_function("lid.generate()", |b| {
        b.iter(|| black_box(lid.generate()));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
