use criterion::async_executor::FuturesExecutor;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kinases::sync::mutex::Mutex;
use std::{
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, available_parallelism},
};

fn use_mutex(mutex: &Mutex<i32>) {
    let mut locked = mutex.blocking_lock();

    *locked += 1;

    assert_eq!(*locked, 1);

    *locked = 0;

    assert_eq!(*locked, 0);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mutex Blocking Lock with threads");

    let machine_thread_count = available_parallelism().map(|n| n.get()).unwrap_or(16);
    for thread_count in [0, 2, 4, machine_thread_count] {
        group.bench_function(
            &format!("mutex locking with {} contention threads", thread_count),
            |b| {
                let m = Arc::new(Mutex::new(0));
                let stop = Arc::new(AtomicBool::new(false));

                let mut thread_joins = Vec::new();
                for _ in 0..thread_count {
                    let m = m.clone();
                    let stop = stop.clone();
                    thread_joins.push(thread::spawn(move || {
                        // Make sure all the threads stay busy for a bit
                        while !stop.load(Ordering::Relaxed) {
                            let mut value = m.blocking_lock();
                            assert_eq!(*value, 0);

                            *value += 1;
                            assert_eq!(*value, 1);

                            *value = 0;
                            assert_eq!(*value, 0);

                            drop(value);
                        }
                    }));
                }

                b.iter_batched(
                    || m.clone(),
                    |m| use_mutex(black_box(&m)),
                    BatchSize::SmallInput,
                );

                stop.store(true, Ordering::SeqCst);
                for thread in thread_joins {
                    thread.join().unwrap();
                }
            },
        );
    }

    group.finish();

    let mut group = c.benchmark_group("Async Mutex Lock with threads");
    for thread_count in [0, 2, 4, machine_thread_count] {
        group.bench_function(
            &format!(
                "async mutex locking with {} contention threads",
                thread_count
            ),
            |b| {
                let m = Arc::new(Mutex::new(0));
                let stop = Arc::new(AtomicBool::new(false));

                let mut thread_joins = Vec::new();
                for _ in 0..thread_count {
                    let m = m.clone();
                    let stop = stop.clone();
                    thread_joins.push(thread::spawn(move || {
                        futures::executor::block_on(async move {
                            while !stop.load(Ordering::Relaxed) {
                                let mut value = m.lock().await;
                                assert_eq!(*value, 0);

                                *value += 1;
                                assert_eq!(*value, 1);

                                *value = 0;
                                assert_eq!(*value, 0);

                                drop(value);
                            }
                        });
                    }));
                }

                b.to_async(FuturesExecutor).iter_batched(
                    || m.clone(),
                    |m| async move {
                        let mut value = m.lock().await;
                        assert_eq!(*value, 0);

                        *value += 1;
                        assert_eq!(*value, 1);

                        *value = 0;
                        assert_eq!(*value, 0);

                        drop(value);
                    },
                    BatchSize::SmallInput,
                );

                stop.store(true, Ordering::SeqCst);
                for thread in thread_joins {
                    thread.join().unwrap();
                }
            },
        );
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
