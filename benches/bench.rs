use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lid::LID;

fn benchmark(c: &mut Criterion) {
    {
        let mut lid = LID::<12, 8>::new();
        c.bench_function(
            "LID w/ 12 prefix, 8 sequence, min incr=100, max incr=1000",
            |b| {
                b.iter(|| black_box(lid.generate()));
            },
        );
    }

    {
        let mut lid = LID::<16, 12>::new();
        c.bench_function(
            "LID w/ 16 prefix, 12 sequence, min incr=100, max incr=1000",
            |b| {
                b.iter(|| black_box(lid.generate()));
            },
        );
    }

    {
        let mut lid = LID::<12, 8, 50_000, 5_000_000>::new();
        c.bench_function(
            "LID w/ 12 prefix, 8 sequence, min incr=50_000, max incr=5_000_000",
            |b| {
                b.iter(|| black_box(lid.generate()));
            },
        );
    }

    {
        let mut lid = LID::<16, 12, 50_000, 5_000_000>::new();
        c.bench_function(
            "LID w/ 16 prefix, 12 sequence, min incr=50_000, max incr=5_000_000",
            |b| {
                b.iter(|| black_box(lid.generate()));
            },
        );
    }

    c.bench_function("colorid w/ 20 bytes", |b| {
        b.iter(|| black_box(colorid::colorid(20)));
    });

    c.bench_function("colorid w/ 28 bytes", |b| {
        b.iter(|| black_box(colorid::colorid(28)));
    });

    c.bench_function("nanoid::nanoid!()", |b| {
        b.iter(|| black_box(nanoid::nanoid!()));
    });

    c.bench_function("nanoid::nanoid!(28)", |b| {
        b.iter(|| black_box(nanoid::nanoid!(28)));
    });

    c.bench_function("snowflaked (i64)", |b| {
        let mut gen = snowflaked::Generator::new(6969);
        b.iter(|| black_box(gen.generate::<i64>()));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
