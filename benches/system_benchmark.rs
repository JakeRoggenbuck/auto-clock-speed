use autoclockspeed::{power, system};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn check_cpu_name_benchmark(c: &mut Criterion) {
    c.bench_function("check_cpu_name", |b| b.iter(system::check_cpu_name));
}

pub fn check_available_governors_benchmark(c: &mut Criterion) {
    c.bench_function("check_available_governors", |b| {
        b.iter(system::check_available_governors)
    });
}

pub fn list_cpus_benchmark(c: &mut Criterion) {
    c.bench_function("list_cpus", |b| b.iter(system::list_cpus));
}

pub fn set_best_path_benchmark(c: &mut Criterion) {
    c.bench_function("set_best_path", |b| b.iter(power::set_best_path));
}

criterion_group!(
    benches,
    check_cpu_name_benchmark,
    check_available_governors_benchmark,
    list_cpus_benchmark,
    set_best_path_benchmark
);

criterion_main!(benches);
