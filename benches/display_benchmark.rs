use autoclockspeed::display::{print_battery_status, print_turbo};
use autoclockspeed::power::battery::{Battery, BatteryStatus};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn print_battery_status_benchmark(c: &mut Criterion) {
    let battery = Battery {
        capacity: 50,
        status: BatteryStatus::Charging,
        ..Default::default()
    };
    c.bench_function("print_battery_status", |b| {
        b.iter(|| black_box(print_battery_status(&battery)))
    });
}

fn print_turbo_benchmark(c: &mut Criterion) {
    c.bench_function("print_turbo", |b| {
        b.iter(|| black_box(print_turbo(true, false)))
    });
}

criterion_group!(
    benches,
    print_battery_status_benchmark,
    print_turbo_benchmark
);
criterion_main!(benches);
