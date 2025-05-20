use autoclockspeed::setup::{inside_docker_message, inside_wsl_message};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn inside_wsl_message_benchmark(c: &mut Criterion) {
    c.bench_function("inside_wsl_message", |b| {
        b.iter(|| black_box(inside_wsl_message()))
    });
}

fn inside_docker_message_benchmark(c: &mut Criterion) {
    c.bench_function("inside_docker_message", |b| {
        b.iter(|| black_box(inside_docker_message()))
    });
}

criterion_group!(
    benches,
    inside_wsl_message_benchmark,
    inside_docker_message_benchmark
);
criterion_main!(benches);
