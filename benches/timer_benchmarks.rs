use criterion::{black_box, criterion_group, criterion_main, Criterion};
use timer_ship::TimerShip;
use std::sync::Arc;

fn benchmark_timer_creation(c: &mut Criterion) {
    let timer_ship = TimerShip::new("bench_test.log").expect("Failed to create TimerShip");
    
    c.bench_function("timer_creation", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            // Use long durations to avoid timers expiring during benchmark
            let duration_str = format!("{}h", (counter % 24) + 1);
            let data = format!("Benchmark timer #{}", counter);
            
            black_box(
                timer_ship.set_timer_with_duration(&duration_str, data)
            ).unwrap();
        })
    });
}

fn benchmark_timer_listing(c: &mut Criterion) {
    let timer_ship = TimerShip::new("bench_list_test.log").expect("Failed to create TimerShip");
    
    // Pre-populate with long-duration timers to ensure they stay active
    for i in 0..1000 {
        let duration_str = format!("{}h", (i % 24) + 1); // 1-24 hours
        let data = format!("List benchmark timer #{}", i);
        let _ = timer_ship.set_timer_with_duration(&duration_str, data);
    }
    
    c.bench_function("timer_listing", |b| {
        b.iter(|| {
            black_box(timer_ship.list_active_timers());
        })
    });
}

fn benchmark_create_remove_cycle(c: &mut Criterion) {
    let timer_ship = TimerShip::new("bench_cycle_test.log").expect("Failed to create TimerShip");
    
    c.bench_function("create_remove_cycle", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            // Create timer with long duration
            let duration_str = format!("{}h", (counter % 24) + 1);
            let data = format!("Cycle benchmark timer #{}", counter);
            
            let timer_id = timer_ship.set_timer_with_duration(&duration_str, data).unwrap();
            // Immediately remove it
            black_box(timer_ship.remove_timer(timer_id).unwrap());
        })
    });
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    c.bench_function("concurrent_timer_creation", |b| {
        b.iter(|| {
            let timer_ship = Arc::new(
                TimerShip::new("bench_concurrent_test.log").expect("Failed to create TimerShip")
            );
            
            let handles: Vec<_> = (0..4).map(|thread_id| {
                let timer_ship = Arc::clone(&timer_ship);
                std::thread::spawn(move || {
                    for i in 0..25 {
                        // Use long durations to avoid expiration during test
                        let duration_str = format!("{}h", (i % 24) + 1);
                        let data = format!("Thread {} timer #{}", thread_id, i);
                        let _ = timer_ship.set_timer_with_duration(&duration_str, data);
                    }
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
            
            black_box(timer_ship.active_timer_count());
        })
    });
}

fn benchmark_recovery(c: &mut Criterion) {
    // Pre-create a log file with many operations using long durations
    {
        let timer_ship = TimerShip::new("bench_recovery_test.log").expect("Failed to create TimerShip");
        for i in 0..500 {
            let duration_str = format!("{}h", (i % 24) + 1); // Long durations
            let data = format!("Recovery benchmark timer #{}", i);
            let _ = timer_ship.set_timer_with_duration(&duration_str, data);
        }
    } // TimerShip goes out of scope
    
    c.bench_function("recovery_from_log", |b| {
        b.iter(|| {
            black_box(
                TimerShip::new("bench_recovery_test.log").expect("Failed to create TimerShip")
            );
        })
    });
}

// Add a benchmark specifically for very short timers (expiration testing)
fn benchmark_short_timer_expiration(c: &mut Criterion) {
    c.bench_function("short_timer_expiration", |b| {
        b.iter(|| {
            let expired_count = Arc::new(std::sync::Mutex::new(0));
            let expired_clone = Arc::clone(&expired_count);
            
            let callback: timer_ship::TimerCallback = Box::new(move |_id, _data| {
                let mut count = expired_clone.lock().unwrap();
                *count += 1;
            });
            
            let timer_ship = TimerShip::with_callback("bench_expiration_test.log", Some(callback))
                .expect("Failed to create TimerShip");
            
            // Create 10 very short timers
            for i in 0..10 {
                let duration_str = format!("{}ms", 50 + (i * 10)); // 50-140ms
                let data = format!("Short timer #{}", i);
                let _ = timer_ship.set_timer_with_duration(&duration_str, data);
            }
            
            // Wait for all to expire
            std::thread::sleep(std::time::Duration::from_millis(200));
            
            black_box(*expired_count.lock().unwrap());
        })
    });
}

criterion_group!(
    benches,
    benchmark_timer_creation,
    benchmark_timer_listing,
    benchmark_create_remove_cycle,
    benchmark_concurrent_operations,
    benchmark_recovery,
    benchmark_short_timer_expiration
);
criterion_main!(benches);
