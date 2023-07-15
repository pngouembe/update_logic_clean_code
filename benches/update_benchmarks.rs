use criterion::{criterion_group, criterion_main, Criterion};

use update_logic_clean_code::{multi_threaded_update, sequencial_update};

fn multi_threaded_update_benchmark(c: &mut Criterion) {
    c.bench_function("multi_threaded_update", |b| {
        b.iter(|| {
            multi_threaded_update(
                "./resources/test/test_lb_cfg.json",
                "./resources/test/update_folder.zip",
            )
        })
    });
}

fn sequencial_update_benchmark(c: &mut Criterion) {
    c.bench_function("sequential_update", |b| {
        b.iter(|| {
            sequencial_update(
                "./resources/test/test_lb_cfg.json",
                "./resources/test/update_folder.zip",
            )
            .unwrap();
        })
    });
}

criterion_group!(
    benches,
    multi_threaded_update_benchmark,
    sequencial_update_benchmark
);
criterion_main!(benches);
