use autoclockspeed::graph::{Graph, Grapher};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn update_all_benchmark(c: &mut Criterion) {
    let mut graph = Graph::new();
    let vec = vec![0.0; 100];
    graph.vals = vec;
    c.bench_function("update_all", |b| b.iter(|| graph.update_all()));
}

fn update_one_benchmark(c: &mut Criterion) {
    let graph = Graph::new();
    let mut vec = vec![0.0; 100];
    c.bench_function("update_one", |b| {
        b.iter(|| black_box(graph.update_one(&mut vec)))
    });
}

fn clear_before_benchmark(c: &mut Criterion) {
    let graph = Graph::new();
    let mut vec = vec![0.0; 100];
    c.bench_function("clear_before", |b| {
        b.iter(|| {
            graph.clear_before(&mut vec);
            black_box(())
        })
    });
}

fn plot_benchmark(c: &mut Criterion) {
    let graph = Graph::new();
    let nums = vec![0.0; 100];
    c.bench_function("plot", |b| b.iter(|| black_box(graph.plot(nums.clone()))));
}

criterion_group!(
    benches,
    update_all_benchmark,
    update_one_benchmark,
    clear_before_benchmark,
    plot_benchmark
);

criterion_main!(benches);
