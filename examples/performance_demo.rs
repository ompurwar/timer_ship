use timer_ship::performance_tests::PerformanceTests;
use std::env;

fn main() {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    println!("🎯 TimerShip Performance Testing Demo");
    println!("=====================================");
    println!("📝 Duration-Aware Performance Testing:");
    println!("   • Creation/Memory tests use LONG durations (hours) to avoid expiration");
    println!("   • Expiration tests use SHORT durations (50-150ms) for quick completion");
    println!("   • Create-remove tests don't wait for expiration");
    println!("   • Recovery tests measure log replay speed, not timer execution\n");

    let perf_tests = PerformanceTests::new();
    
    // Check command line arguments for specific tests
    let args: Vec<String> = env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("throughput") => {
            println!("Running throughput test...");
            let result = perf_tests.test_timer_creation_throughput(5000);
            result.print_summary();
        },
        Some("concurrent") => {
            println!("Running concurrent operations test...");
            let result = perf_tests.test_concurrent_operations(8, 500);
            result.print_summary();
        },
        Some("recovery") => {
            println!("Running recovery performance test...");
            let result = perf_tests.test_recovery_performance(2000);
            result.print_summary();
        },
        Some("expiration") => {
            println!("Running expiration performance test...");
            let result = perf_tests.test_expiration_performance(500);
            result.print_summary();
        },
        Some("memory") => {
            println!("Running memory scalability test...");
            let results = perf_tests.test_memory_scalability(vec![500, 1000, 2000, 5000]);
            for result in results {
                result.print_summary();
            }
        },
        Some("all") | None => {
            perf_tests.run_all_tests();
        },
        Some(unknown) => {
            println!("❌ Unknown test: {}", unknown);
            println!("Available tests: throughput, concurrent, recovery, expiration, memory, all");
        }
    }
}
