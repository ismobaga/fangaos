//! Performance Profiling
//!
//! This module provides built-in performance profiling tools including:
//! - Sampling-based profiler
//! - Performance counter integration
//! - Call stack sampling
//! - Performance statistics

pub mod sampler;
pub mod pmu;
pub mod stats;
pub mod output;

pub use sampler::{Profiler, ProfileSample, SamplingConfig};
pub use pmu::{PerformanceCounter, CounterType, CounterEvent};
pub use stats::{ProfileStats, FunctionStats};
pub use output::{ProfileOutput, OutputFormat};

use spin::Once;

/// Global profiler instance
static PROFILER: Once<spin::Mutex<Profiler>> = Once::new();

/// Initialize the profiling subsystem
pub fn init() -> Result<(), &'static str> {
    let profiler = Profiler::new();
    PROFILER.call_once(|| spin::Mutex::new(profiler));
    
    Ok(())
}

/// Get reference to the profiler
pub fn profiler() -> &'static spin::Mutex<Profiler> {
    PROFILER.get().expect("Profiler not initialized")
}

/// Start profiling
pub fn start_profiling() -> Result<(), &'static str> {
    let mut prof = profiler().lock();
    prof.start()
}

/// Stop profiling
pub fn stop_profiling() -> Result<(), &'static str> {
    let mut prof = profiler().lock();
    prof.stop()
}

/// Get profiling statistics
pub fn get_stats() -> ProfileStats {
    let prof = profiler().lock();
    prof.get_stats()
}
