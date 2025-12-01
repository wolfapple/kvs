
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::prelude::*;
use tempfile::TempDir;

fn write_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");
    for i in [1, 10, 100, 500, 1000].iter() {
        let size = i * 1024;
        let mut rng = SmallRng::seed_from_u64(42);
        let key = format!("key_{}", rng.random_range(0..100000));
        let value: String = rng
            .sample_iter(&rand::distr::Alphanumeric)
            .take(size)
            .map(char::from)
            .collect();

        group.bench_with_input(BenchmarkId::new("kvs", size), &value, |b, value| {
            let temp_dir = TempDir::new().unwrap();
            let store = KvStore::open(temp_dir.path()).unwrap();
            b.iter(|| {
                store.set(key.clone(), value.clone()).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("sled", size), &value, |b, value| {
            let temp_dir = TempDir::new().unwrap();
            let store = SledKvsEngine::open(temp_dir.path()).unwrap();
            b.iter(|| {
                store.set(key.clone(), value.clone()).unwrap();
            })
        });
    }
    group.finish();
}

fn read_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    for i in [1, 10, 100, 500, 1000].iter() {
        let size = i * 1024;
        let mut rng = SmallRng::seed_from_u64(42);
        let key = format!("key_{}", rng.random_range(0..100000));
        let value: String = rng
            .sample_iter(&rand::distr::Alphanumeric)
            .take(size)
            .map(char::from)
            .collect();

        group.bench_with_input(BenchmarkId::new("kvs", size), &value, |b, value| {
            let temp_dir = TempDir::new().unwrap();
            let store = KvStore::open(temp_dir.path()).unwrap();
            store.set(key.clone(), value.clone()).unwrap();
            b.iter(|| {
                store.get(key.clone()).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("sled", size), &value, |b, value| {
            let temp_dir = TempDir::new().unwrap();
            let store = SledKvsEngine::open(temp_dir.path()).unwrap();
            store.set(key.clone(), value.clone()).unwrap();
            b.iter(|| {
                store.get(key.clone()).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, write_benchmark, read_benchmark);
criterion_main!(benches);
