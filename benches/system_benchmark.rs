use autoclockspeed::system;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn check_cpu_name_benchmark(c: &mut Criterion) {
    c.bench_function("check_cpu_name", |b| b.iter(|| system::check_cpu_name()));
}

pub fn check_available_governors_benchmark(c: &mut Criterion) {
    c.bench_function("check_available_governors", |b| {
        b.iter(|| system::check_available_governors())
    });
}

criterion_group!(
    benches,
    check_cpu_name_benchmark,
    check_available_governors_benchmark
);
criterion_main!(benches);
