use async_once::AsyncOnce;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use std::ops::Deref;
use tokio::runtime::Runtime;

lazy_static! {
    static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
}

lazy_static! {
    static ref FOO_SYNC: u32 = {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async { 1 })
    };
}

fn async_once_benchmark(c: &mut Criterion) {
    let mut rt = Runtime::new().unwrap();
    c.bench_function("async once", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..100000 {
                    black_box(FOO.get().await);
                }
            });
        })
    });
}

fn sync_once_benchmark(c: &mut Criterion) {
    let mut rt = Runtime::new().unwrap();
    c.bench_function("sync once", |b| {
        b.iter(|| {
            let _ = FOO_SYNC.deref();
            rt.block_on(async {
                for _ in 0..100000 {
                    black_box(async { FOO_SYNC.deref() == &1 }.await);
                }
            });
        })
    });
}

criterion_group!(benches, async_once_benchmark, sync_once_benchmark);
criterion_main!(benches);
