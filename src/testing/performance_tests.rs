use crate::{TimerShip, TimerCallback};
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

fn setup_timer_ship() -> TimerShip {
    // Create TimerShip with a dummy log path and no-op callback
    let callback: TimerCallback = Box::new(|_, _| {});
    TimerShip::with_callback("dummy_log_path.log", Some(callback)).unwrap()
}

fn bench_set_timer(c: &mut Criterion) {
    c.bench_function("set_timer", |b| {
        let timer_ship = setup_timer_ship();
        b.iter(|| {
            let _ = timer_ship.set_timer_with_duration("5s", "Test timer".to_string());
        })
    });
}

fn bench_list_active_timers(c: &mut Criterion) {
    c.bench_function("list_active_timers", |b| {
        let timer_ship = setup_timer_ship();
        
        // Pre-populate with timers
        for i in 0..100 {
            let _ = timer_ship.set_timer_with_duration("5s", format!("Test timer {}", i));
        }
        
        b.iter(|| {
            let _ = timer_ship.list_active_timers();
        })
    });
}

fn bench_remove_timer(c: &mut Criterion) {
    c.bench_function("remove_timer", |b| {
        let timer_ship = setup_timer_ship();
        
        // Pre-populate with a timer to remove
        let timer_id = timer_ship.set_timer_with_duration("5s", "Timer to be removed".to_string()).unwrap();
        
        b.iter(|| {
            let _ = timer_ship.remove_timer(timer_id);
        })
    });
}

criterion_group!(benches, bench_set_timer, bench_list_active_timers, bench_remove_timer);
criterion_main!(benches);