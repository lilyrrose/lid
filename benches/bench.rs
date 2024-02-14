use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lid::LID;

fn benchmark(c: &mut Criterion) {
    {
        let mut lid = LID::<12, 8>::new();
        c.bench_function("lid.generate() (20 byte id)", |b| {
            b.iter(|| black_box(lid.generate()));
        });
    }

    {
        let mut lid = LID::<16, 12>::new();
        c.bench_function("lid.generate() (28 byte id)", |b| {
            b.iter(|| black_box(lid.generate()));
        });
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
