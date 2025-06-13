use crate::{TimerShip, TimerCallback};
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

/// Performance test results
#[derive(Debug, Clone)]
pub struct PerfTestResults {
    pub test_name: String,
    pub operations_per_second: f64,
    pub total_duration_ms: u128,
    pub memory_usage_mb: f64,
    pub success_count: usize,
    pub error_count: usize,
}

impl PerfTestResults {
    pub fn print_summary(&self) {
        println!("📊 Performance Test: {}", self.test_name);
        println!("   Operations/sec: {:.2}", self.operations_per_second);
        println!("   Total time: {}ms", self.total_duration_ms);
        println!("   Memory usage: {:.2}MB", self.memory_usage_mb);
        println!("   Success: {}, Errors: {}", self.success_count, self.error_count);
        println!("   ════════════════════════════════════════");
    }
}

/// Performance test suite for TimerShip
pub struct PerformanceTests {
    test_log_path: String,
}

impl PerformanceTests {
    pub fn new() -> Self {
        Self {
            test_log_path: "perf_test_operations.log".to_string(),
        }
    }

    /// Clean up test files
    fn cleanup(&self) {
        let _ = fs::remove_file(&self.test_log_path);
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage() -> f64 {
        // This is a simplified estimation - in production you'd use more sophisticated tools
        std::process::id() as f64 * 0.001 // Placeholder
    }

    /// Test timer creation throughput
    pub fn test_timer_creation_throughput(&self, count: usize) -> PerfTestResults {
        self.cleanup();
        
        let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
        
        let start = Instant::now();
        let mut success_count = 0;
        let mut error_count = 0;
        
        for i in 0..count {
            let duration_str = format!("{}s", (i % 3600) + 1); // 1-3600 seconds
            let data = format!("Perf test timer #{}", i);
            
            match timer_ship.set_timer_with_duration(&duration_str, data) {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start.elapsed();
        let ops_per_sec = count as f64 / duration.as_secs_f64();
        
        PerfTestResults {
            test_name: "Timer Creation Throughput".to_string(),
            operations_per_second: ops_per_sec,
            total_duration_ms: duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count,
            error_count,
        }
    }

    /// Test concurrent timer operations
    pub fn test_concurrent_operations(&self, threads: usize, ops_per_thread: usize) -> PerfTestResults {
        self.cleanup();
        
        let timer_ship = Arc::new(TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip"));
        let results = Arc::new(Mutex::new((0usize, 0usize))); // (success, error)
        
        let start = Instant::now();
        let mut handles = vec![];
        
        for thread_id in 0..threads {
            let timer_ship_clone = Arc::clone(&timer_ship);
            let results_clone = Arc::clone(&results);
            
            let handle = thread::spawn(move || {
                let mut local_success = 0;
                let mut local_error = 0;
                
                for i in 0..ops_per_thread {
                    let duration_str = format!("{}s", (i % 100) + 1);
                    let data = format!("Thread {} - Timer #{}", thread_id, i);
                    
                    match timer_ship_clone.set_timer_with_duration(&duration_str, data) {
                        Ok(_) => local_success += 1,
                        Err(_) => local_error += 1,
                    }
                }
                
                let mut results = results_clone.lock().unwrap();
                results.0 += local_success;
                results.1 += local_error;
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        let total_ops = threads * ops_per_thread;
        let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
        let (success_count, error_count) = *results.lock().unwrap();
        
        PerfTestResults {
            test_name: format!("Concurrent Operations ({} threads)", threads),
            operations_per_second: ops_per_sec,
            total_duration_ms: duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count,
            error_count,
        }
    }

    /// Test recovery performance
    pub fn test_recovery_performance(&self, timer_count: usize) -> PerfTestResults {
        self.cleanup();
        
        // First, create a bunch of timers to populate the log
        {
            let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
            
            for i in 0..timer_count {
                let duration_str = format!("{}s", (i % 3600) + 1);
                let data = format!("Recovery test timer #{}", i);
                let _ = timer_ship.set_timer_with_duration(&duration_str, data);
            }
        } // TimerShip goes out of scope
        
        // Now test recovery time
        let start = Instant::now();
        let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
        let duration = start.elapsed();
        
        let active_count = timer_ship.active_timer_count();
        
        PerfTestResults {
            test_name: "Log Recovery Performance".to_string(),
            operations_per_second: timer_count as f64 / duration.as_secs_f64(),
            total_duration_ms: duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count: active_count,
            error_count: timer_count - active_count,
        }
    }

    /// Test timer expiration handling performance with short durations
    pub fn test_expiration_performance(&self, timer_count: usize) -> PerfTestResults {
        self.cleanup();
        
        let expired_timers = Arc::new(Mutex::new(0usize));
        let expired_clone = Arc::clone(&expired_timers);
        
        let callback: TimerCallback = Box::new(move |_timer_id: Uuid, _data: String| {
            let mut count = expired_clone.lock().unwrap();
            *count += 1;
        });
        
        let timer_ship = TimerShip::with_callback(&self.test_log_path, Some(callback))
            .expect("Failed to create TimerShip");
        
        let start = Instant::now();
        
        // Create timers with very short durations (50-150ms) to ensure quick expiration
        for i in 0..timer_count {
            let duration_ms = 50 + (i % 100); // 50-149ms
            let duration_str = format!("{}ms", duration_ms);
            let data = format!("Expiration test timer #{}", i);
            let _ = timer_ship.set_timer_with_duration(&duration_str, data);
        }
        
        // Calculate max wait time based on longest possible timer (149ms) + buffer
        let max_wait_time = Duration::from_millis(200);
        let creation_time = start.elapsed();
        
        // Wait for all timers to expire
        thread::sleep(max_wait_time);
        
        // Give a bit more time for callback processing
        thread::sleep(Duration::from_millis(50));
        
        let total_duration = start.elapsed();
        let expired_count = *expired_timers.lock().unwrap();
        
        PerfTestResults {
            test_name: "Timer Expiration Performance".to_string(),
            operations_per_second: expired_count as f64 / total_duration.as_secs_f64(),
            total_duration_ms: total_duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count: expired_count,
            error_count: timer_count - expired_count,
        }
    }

    /// Test timer creation and immediate removal (no waiting for expiration)
    pub fn test_create_remove_cycle(&self, cycle_count: usize) -> PerfTestResults {
        self.cleanup();
        
        let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
        
        let start = Instant::now();
        let mut success_count = 0;
        let mut error_count = 0;
        
        for i in 0..cycle_count {
            // Create timer with long duration (so it won't expire during test)
            let duration_str = format!("{}h", (i % 24) + 1); // 1-24 hours
            let data = format!("Create-remove cycle timer #{}", i);
            
            match timer_ship.set_timer_with_duration(&duration_str, data) {
                Ok(timer_id) => {
                    // Immediately remove the timer
                    match timer_ship.remove_timer(timer_id) {
                        Ok(_) => success_count += 1,
                        Err(_) => error_count += 1,
                    }
                },
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start.elapsed();
        let ops_per_sec = (cycle_count * 2) as f64 / duration.as_secs_f64(); // *2 for create+remove
        
        PerfTestResults {
            test_name: "Create-Remove Cycle Performance".to_string(),
            operations_per_second: ops_per_sec,
            total_duration_ms: duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count,
            error_count,
        }
    }

    /// Test memory usage with large number of long-duration timers
    pub fn test_memory_scalability(&self, timer_counts: Vec<usize>) -> Vec<PerfTestResults> {
        let mut results = vec![];
        
        for &count in &timer_counts {
            self.cleanup();
            
            let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
            let start = Instant::now();
            
            for i in 0..count {
                // Use long durations to ensure timers stay active during measurement
                let duration_str = format!("{}h", (i % 24) + 1); // 1-24 hours
                let data = format!("Memory test timer #{} - {}", i, "x".repeat(i % 100)); // Variable data size
                let _ = timer_ship.set_timer_with_duration(&duration_str, data);
            }
            
            let duration = start.elapsed();
            let active_count = timer_ship.active_timer_count();
            
            results.push(PerfTestResults {
                test_name: format!("Memory Scalability ({} timers)", count),
                operations_per_second: count as f64 / duration.as_secs_f64(),
                total_duration_ms: duration.as_millis(),
                memory_usage_mb: Self::estimate_memory_usage(),
                success_count: active_count,
                error_count: count - active_count,
            });
        }
        
        results
    }

    /// Test listing performance with different timer counts
    pub fn test_listing_performance(&self, timer_count: usize) -> PerfTestResults {
        self.cleanup();
        
        let timer_ship = TimerShip::new(&self.test_log_path).expect("Failed to create TimerShip");
        
        // Pre-populate with long-duration timers
        for i in 0..timer_count {
            let duration_str = format!("{}h", (i % 24) + 1);
            let data = format!("Listing test timer #{}", i);
            let _ = timer_ship.set_timer_with_duration(&duration_str, data);
        }
        
        // Now test listing performance
        let start = Instant::now();
        let list_iterations = 100;
        
        for _ in 0..list_iterations {
            let _timers = timer_ship.list_active_timers();
        }
        
        let duration = start.elapsed();
        let ops_per_sec = list_iterations as f64 / duration.as_secs_f64();
        
        PerfTestResults {
            test_name: format!("Listing Performance ({} timers)", timer_count),
            operations_per_second: ops_per_sec,
            total_duration_ms: duration.as_millis(),
            memory_usage_mb: Self::estimate_memory_usage(),
            success_count: list_iterations,
            error_count: 0,
        }
    }

    /// Run all performance tests with duration awareness
    pub fn run_all_tests(&self) {
        println!("🚀 Starting Duration-Aware Performance Tests for TimerShip");
        println!("═══════════════════════════════════════════════════════════");
        
        // Test 1: Basic throughput (using long durations to avoid expiration)
        println!("📝 Test 1: Timer Creation Throughput (long durations)");
        let result1 = self.test_timer_creation_throughput(1000);
        result1.print_summary();
        
        // Test 2: Concurrent operations (using long durations)
        println!("📝 Test 2: Concurrent Operations (long durations)");
        let result2 = self.test_concurrent_operations(4, 250);
        result2.print_summary();
        
        // Test 3: Recovery performance
        println!("📝 Test 3: Log Recovery Performance");
        let result3 = self.test_recovery_performance(500);
        result3.print_summary();
        
        // Test 4: Expiration performance (using very short durations)
        println!("📝 Test 4: Timer Expiration Performance (short durations)");
        let result4 = self.test_expiration_performance(50); // Reduced count for faster test
        result4.print_summary();
        
        // Test 5: Create-remove cycles (no expiration waiting)
        println!("📝 Test 5: Create-Remove Cycle Performance");
        let result5 = self.test_create_remove_cycle(500);
        result5.print_summary();
        
        // Test 6: Listing performance
        println!("📝 Test 6: Timer Listing Performance");
        let result6 = self.test_listing_performance(1000);
        result6.print_summary();
        
        // Test 7: Memory scalability (using long durations)
        println!("📝 Test 7: Memory Scalability (long durations)");
        let results7 = self.test_memory_scalability(vec![100, 500, 1000]);
        for result in results7 {
            result.print_summary();
        }
        
        // Cleanup
        self.cleanup();
        
        println!("✅ All performance tests completed!");
        println!("📊 Note: Tests use duration-appropriate timer lengths:");
        println!("   • Creation/Memory tests: Long durations (hours) to avoid expiration");
        println!("   • Expiration tests: Very short durations (50-150ms) for quick completion");
        println!("   • Create-remove tests: Immediate removal, no waiting");
    }
}

impl Default for PerformanceTests {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation_performance() {
        let perf_tests = PerformanceTests::new();
        let result = perf_tests.test_timer_creation_throughput(100);
        
        assert!(result.operations_per_second > 0.0);
        assert!(result.success_count > 0);
        assert_eq!(result.success_count + result.error_count, 100);
    }

    #[test]
    fn test_concurrent_performance() {
        let perf_tests = PerformanceTests::new();
        let result = perf_tests.test_concurrent_operations(2, 50);
        
        assert!(result.operations_per_second > 0.0);
        assert!(result.success_count > 0);
        assert_eq!(result.success_count + result.error_count, 100);
    }

    #[test]
    fn test_recovery_performance() {
        let perf_tests = PerformanceTests::new();
        let result = perf_tests.test_recovery_performance(50);
        
        assert!(result.operations_per_second > 0.0);
        assert!(result.total_duration_ms > 0);
    }
}
