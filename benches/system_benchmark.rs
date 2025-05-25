use autoclockspeed::cpu::CPU;
use autoclockspeed::proc::ProcStat;
use autoclockspeed::system::{
    calculate_cpu_percent, check_available_governors, check_cpu_freq, check_cpu_name,
    check_cpu_temperature, check_cpu_usage, check_turbo_enabled, get_cpu_percent, get_highest_temp,
    inside_docker, inside_wsl, list_cpu_governors, list_cpu_speeds, list_cpu_temp, list_cpus,
    read_int, read_str,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn check_cpu_freq_benchmark(c: &mut Criterion) {
    let cpus = vec![CPU::default(), CPU::default()];
    c.bench_function("check_cpu_freq", |b| {
        b.iter(|| black_box(check_cpu_freq(&cpus)))
    });
}

fn check_cpu_usage_benchmark(c: &mut Criterion) {
    let cpus = vec![CPU::default(), CPU::default()];
    c.bench_function("check_cpu_usage", |b| {
        b.iter(|| black_box(check_cpu_usage(&cpus)))
    });
}

fn check_cpu_temperature_benchmark(c: &mut Criterion) {
    let cpus = vec![CPU::default(), CPU::default()];
    c.bench_function("check_cpu_temperature", |b| {
        b.iter(|| black_box(check_cpu_temperature(&cpus)))
    });
}

fn get_highest_temp_benchmark(c: &mut Criterion) {
    let cpus = vec![CPU::default(), CPU::default()];
    c.bench_function("get_highest_temp", |b| {
        b.iter(|| black_box(get_highest_temp(&cpus)))
    });
}

fn inside_docker_benchmark(c: &mut Criterion) {
    c.bench_function("inside_docker", |b| b.iter(|| black_box(inside_docker())));
}

fn inside_wsl_benchmark(c: &mut Criterion) {
    c.bench_function("inside_wsl", |b| b.iter(|| black_box(inside_wsl())));
}

fn check_cpu_name_benchmark(c: &mut Criterion) {
    c.bench_function("check_cpu_name", |b| b.iter(|| black_box(check_cpu_name())));
}

fn get_cpu_percent_benchmark(c: &mut Criterion) {
    c.bench_function("get_cpu_percent", |b| {
        b.iter(|| black_box(get_cpu_percent(None)))
    });
}

fn calculate_cpu_percent_benchmark(c: &mut Criterion) {
    let timing_1 = ProcStat::default();
    let timing_2 = ProcStat::default();
    c.bench_function("calculate_cpu_percent", |b| {
        b.iter(|| black_box(calculate_cpu_percent(&timing_1, &timing_2)))
    });
}

fn check_turbo_enabled_benchmark(c: &mut Criterion) {
    c.bench_function("check_turbo_enabled", |b| {
        b.iter(|| black_box(check_turbo_enabled()))
    });
}

fn check_available_governors_benchmark(c: &mut Criterion) {
    c.bench_function("check_available_governors", |b| {
        b.iter(|| black_box(check_available_governors()))
    });
}

fn list_cpus_benchmark(c: &mut Criterion) {
    c.bench_function("list_cpus", |b| b.iter(|| black_box(list_cpus())));
}

fn list_cpu_speeds_benchmark(c: &mut Criterion) {
    c.bench_function("list_cpu_speeds", |b| {
        b.iter(|| black_box(list_cpu_speeds()))
    });
}

fn list_cpu_temp_benchmark(c: &mut Criterion) {
    c.bench_function("list_cpu_temp", |b| b.iter(|| black_box(list_cpu_temp())));
}

fn list_cpu_governors_benchmark(c: &mut Criterion) {
    c.bench_function("list_cpu_governors", |b| {
        b.iter(|| black_box(list_cpu_governors()))
    });
}

fn read_int_benchmark(c: &mut Criterion) {
    c.bench_function("read_int", |b| {
        b.iter(|| {
            black_box(read_int(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq",
            ))
        })
    });
}

fn read_str_benchmark(c: &mut Criterion) {
    c.bench_function("read_str", |b| {
        b.iter(|| {
            black_box(read_str(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
            ))
        })
    });
}

criterion_group!(
    benches,
    check_cpu_freq_benchmark,
    check_cpu_usage_benchmark,
    check_cpu_temperature_benchmark,
    get_highest_temp_benchmark,
    inside_docker_benchmark,
    inside_wsl_benchmark,
    check_cpu_name_benchmark,
    get_cpu_percent_benchmark,
    calculate_cpu_percent_benchmark,
    check_turbo_enabled_benchmark,
    check_available_governors_benchmark,
    list_cpus_benchmark,
    list_cpu_speeds_benchmark,
    list_cpu_temp_benchmark,
    list_cpu_governors_benchmark,
    read_int_benchmark,
    read_str_benchmark
);
criterion_main!(benches);
