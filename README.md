# Timer Utility - Persistent Timer System with Operation Logging

A robust, persistent timer utility written in Rust that provides reliable timer management with automatic recovery capabilities through operation logging.

## What is this utility?

This timer utility solves the common problem of timer persistence and reliability in applications that need to schedule tasks or events. Unlike simple in-memory timers that are lost when an application crashes or restarts, this utility ensures that all timers are persisted and can be recovered automatically.

## Why does this utility exist?

### Problem Statement

Many applications need to schedule tasks or events to happen at specific times or after certain durations. Common scenarios include:

- **Reminder systems**: Schedule notifications for users
- **Task scheduling**: Execute delayed operations
- **Session timeouts**: Clean up expired sessions
- **Retry mechanisms**: Retry failed operations after delays
- **Rate limiting**: Reset counters after time windows
- **Cache expiration**: Remove stale data automatically

The challenge with traditional timer implementations is that they are vulnerable to:
- **Application crashes**: In-memory timers are lost
- **System restarts**: All scheduled timers disappear
- **Process kills**: Timers never fire
- **Deployment restarts**: Scheduled events are forgotten

### Solution Approach

This utility addresses these problems by:

1. **Persistent Storage**: All timer operations are logged to disk using an append-only log
2. **Automatic Recovery**: On startup, the system replays all logged operations to restore timer state
3. **Crash Resilience**: Even if the application crashes, timers are recovered on restart
4. **Human-Readable Duration Format**: Easy-to-use API with duration strings like "1.5m", "30s", "2hr"
5. **UUID-based IDs**: Globally unique timer identifiers for reliable tracking
6. **Millisecond Precision**: High-precision timing for accurate scheduling

## Key Features

### 🔄 **Persistent Operation Logging**
- Every timer operation (create/delete) is logged to an append-only file
- Uses JSON serialization for human-readable logs
- Automatic recovery by replaying operation logs on startup

### ⏱️ **High-Precision Timing**
- Internal millisecond precision for accurate timing
- Human-readable duration strings: `"1ms"`, `"10s"`, `"1.5m"`, `"2hr"`
- Support for floating-point durations (except milliseconds)

### 🆔 **UUID-based Timer IDs**
- Globally unique identifiers for each timer
- No ID collisions even across application restarts
- Reliable timer tracking and management

### 🧵 **Concurrent Processing**
- Background thread handles timer expiration
- Non-blocking API for setting and removing timers
- Thread-safe operations with proper mutex handling

### 📊 **Comprehensive Logging**
- Structured logging using the `log` crate
- Configurable log levels (debug, info, warn, error)
- Detailed recovery and operation tracking

## Use Cases

### 1. **Web Application Session Management**
```rust
// Set session timeout for 30 minutes
let timer_id = timer_ship.set_timer_with_duration("30m", format!("session:{}", user_id))?;

// On user activity, remove old timer and set new one
timer_ship.remove_timer(timer_id)?;
let new_timer_id = timer_ship.set_timer_with_duration("30m", format!("session:{}", user_id))?;
```

### 2. **Reminder System**
```rust
// Schedule reminder for 2 hours from now
let reminder_id = timer_ship.set_timer_with_duration("2hr", "Doctor appointment reminder".to_string())?;
```

### 3. **Cache Expiration**
```rust
// Cache entry expires in 5 minutes
let cache_timer = timer_ship.set_timer_with_duration("5m", format!("cache_key:{}", key))?;
```

### 4. **Retry Mechanism**
```rust
// Retry failed operation after 30 seconds
let retry_timer = timer_ship.set_timer_with_duration("30s", format!("retry_task:{}", task_id))?;
```

### 5. **Rate Limiting**
```rust
// Reset rate limit counter after 1 hour
let reset_timer = timer_ship.set_timer_with_duration("1hr", format!("rate_limit_reset:{}", user_id))?;
```

## Duration Format

The utility supports intuitive duration strings:

| Format | Description | Example |
|--------|-------------|---------|
| `"Xms"` | Milliseconds (integer only) | `"100ms"`, `"1500ms"` |
| `"Xs"` | Seconds (float supported) | `"1s"`, `"2.5s"`, `"30s"` |
| `"Xm"` | Minutes (float supported) | `"1m"`, `"1.5m"`, `"45m"` |
| `"Xh"` or `"Xhr"` | Hours (float supported) | `"1h"`, `"2.5hr"`, `"24hr"` |

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │───▶│   TimerShip API  │───▶│  Operation Log  │
│     Code        │    │                  │    │   (JSON File)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │ Timer Processing │
                       │     Thread       │
                       └──────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │ Timer Expiration │
                       │    Callbacks     │
                       └──────────────────┘
```

## Recovery Process

1. **Startup**: Application starts and creates TimerShip instance
2. **Log Reading**: System reads all entries from the operation log file
3. **State Reconstruction**: Replays all operations to rebuild timer state
4. **Background Thread**: Starts timer processing thread
5. **Normal Operation**: API becomes available for new timer operations

## Safety Guarantees

- **Crash Recovery**: All timers survive application crashes
- **Atomicity**: Timer operations are logged before being applied
- **Consistency**: Recovery process ensures consistent state
- **Durability**: Operations are flushed to disk immediately
- **Thread Safety**: All operations are protected by mutexes

## When to Use This Utility

✅ **Good for:**
- Applications that need persistent timers
- Systems that require crash recovery
- Microservices with timer-based logic
- Long-running background processes
- Applications with scheduled tasks

❌ **Not ideal for:**
- High-frequency, short-duration timers (< 100ms)
- Applications that restart frequently (due to recovery overhead)
- Systems where timer precision is critical (network timing, real-time systems)
- Simple applications that don't need persistence

## Getting Started

### Basic Usage
```rust
use timer_util::TimerShip;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create timer system with persistent log
    let timer_ship = TimerShip::new("timers.log")?;
    
    // Schedule a timer for 30 seconds
    let timer_id = timer_ship.set_timer_with_duration("30s", "My task".to_string())?;
    
    // Timer will fire even if application restarts!
    Ok(())
}
```

### With Expiration Callbacks
```rust
use timer_util::{TimerShip, TimerCallback};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create callback for timer expiration
    let callback: TimerCallback = Box::new(|timer_id: Uuid, data: String| {
        println!("Timer {} expired with data: {}", timer_id, data);
        
        // Execute your custom logic here:
        // - Send notifications
        // - Clean up resources  
        // - Trigger other operations
    });
    
    // Create timer system with callback
    let timer_ship = TimerShip::with_callback("timers.log", Some(callback))?;
    
    // Schedule a timer - callback will be called when it expires
    let timer_id = timer_ship.set_timer_with_duration("30s", "My task".to_string())?;
    
    Ok(())
}
```

This utility provides a robust foundation for any application that needs reliable, persistent timer functionality with automatic recovery capabilities.
