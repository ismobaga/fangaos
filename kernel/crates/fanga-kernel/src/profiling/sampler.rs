//! Sampling-based Profiler

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Sampling configuration
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Sampling interval in microseconds
    pub interval_us: u64,
    
    /// Maximum number of samples to collect
    pub max_samples: usize,
    
    /// Include kernel samples
    pub include_kernel: bool,
    
    /// Include user samples
    pub include_user: bool,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            interval_us: 1000, // 1ms
            max_samples: 10000,
            include_kernel: true,
            include_user: true,
        }
    }
}

/// A single profile sample
#[derive(Debug, Clone)]
pub struct ProfileSample {
    /// Timestamp (in ticks)
    pub timestamp: u64,
    
    /// Instruction pointer
    pub rip: u64,
    
    /// Stack pointer
    pub rsp: u64,
    
    /// CPU ID
    pub cpu_id: usize,
    
    /// Task ID
    pub task_id: usize,
    
    /// Was this in kernel mode?
    pub kernel_mode: bool,
}

/// Profiler state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilerState {
    /// Profiler is stopped
    Stopped,
    
    /// Profiler is running
    Running,
    
    /// Profiler is paused
    Paused,
}

/// Sampling-based profiler
pub struct Profiler {
    /// Current state
    state: ProfilerState,
    
    /// Configuration
    config: SamplingConfig,
    
    /// Collected samples
    samples: Vec<ProfileSample>,
    
    /// Sample count by instruction pointer
    sample_counts: BTreeMap<u64, usize>,
    
    /// Total samples collected
    total_samples: usize,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            state: ProfilerState::Stopped,
            config: SamplingConfig::default(),
            samples: Vec::new(),
            sample_counts: BTreeMap::new(),
            total_samples: 0,
        }
    }
    
    /// Configure the profiler
    pub fn configure(&mut self, config: SamplingConfig) {
        self.config = config;
    }
    
    /// Start profiling
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state == ProfilerState::Running {
            return Err("Profiler already running");
        }
        
        self.state = ProfilerState::Running;
        self.samples.clear();
        self.sample_counts.clear();
        self.total_samples = 0;
        
        // TODO: Set up timer for sampling
        
        Ok(())
    }
    
    /// Stop profiling
    pub fn stop(&mut self) -> Result<(), &'static str> {
        if self.state != ProfilerState::Running {
            return Err("Profiler not running");
        }
        
        self.state = ProfilerState::Stopped;
        
        // TODO: Disable sampling timer
        
        Ok(())
    }
    
    /// Pause profiling
    pub fn pause(&mut self) -> Result<(), &'static str> {
        if self.state != ProfilerState::Running {
            return Err("Profiler not running");
        }
        
        self.state = ProfilerState::Paused;
        Ok(())
    }
    
    /// Resume profiling
    pub fn resume(&mut self) -> Result<(), &'static str> {
        if self.state != ProfilerState::Paused {
            return Err("Profiler not paused");
        }
        
        self.state = ProfilerState::Running;
        Ok(())
    }
    
    /// Record a sample
    pub fn record_sample(&mut self, sample: ProfileSample) {
        if self.state != ProfilerState::Running {
            return;
        }
        
        if self.samples.len() >= self.config.max_samples {
            return; // Drop sample if at capacity
        }
        
        // Update sample count for this instruction pointer
        *self.sample_counts.entry(sample.rip).or_insert(0) += 1;
        
        self.samples.push(sample);
        self.total_samples += 1;
    }
    
    /// Get current state
    pub fn state(&self) -> ProfilerState {
        self.state
    }
    
    /// Get total number of samples
    pub fn sample_count(&self) -> usize {
        self.total_samples
    }
    
    /// Get samples
    pub fn samples(&self) -> &[ProfileSample] {
        &self.samples
    }
    
    /// Get sample counts by instruction pointer
    pub fn sample_counts(&self) -> &BTreeMap<u64, usize> {
        &self.sample_counts
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> super::ProfileStats {
        super::ProfileStats::from_profiler(self)
    }
    
    /// Get top N hotspots (most sampled instruction pointers)
    pub fn top_hotspots(&self, n: usize) -> Vec<(u64, usize)> {
        let mut hotspots: Vec<_> = self.sample_counts.iter()
            .map(|(&rip, &count)| (rip, count))
            .collect();
        
        hotspots.sort_by(|a, b| b.1.cmp(&a.1));
        hotspots.truncate(n);
        hotspots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new();
        assert_eq!(profiler.state(), ProfilerState::Stopped);
        assert_eq!(profiler.sample_count(), 0);
    }
    
    #[test]
    fn test_profiler_start_stop() {
        let mut profiler = Profiler::new();
        
        profiler.start().unwrap();
        assert_eq!(profiler.state(), ProfilerState::Running);
        
        profiler.stop().unwrap();
        assert_eq!(profiler.state(), ProfilerState::Stopped);
    }
    
    #[test]
    fn test_profiler_record_sample() {
        let mut profiler = Profiler::new();
        profiler.start().unwrap();
        
        let sample = ProfileSample {
            timestamp: 1000,
            rip: 0x1000,
            rsp: 0x2000,
            cpu_id: 0,
            task_id: 1,
            kernel_mode: true,
        };
        
        profiler.record_sample(sample);
        assert_eq!(profiler.sample_count(), 1);
    }
    
    #[test]
    fn test_profiler_hotspots() {
        let mut profiler = Profiler::new();
        profiler.start().unwrap();
        
        // Record samples at different IPs
        for i in 0..10 {
            let sample = ProfileSample {
                timestamp: i,
                rip: 0x1000 + (i % 3) * 0x10, // 3 different IPs
                rsp: 0x2000,
                cpu_id: 0,
                task_id: 1,
                kernel_mode: true,
            };
            profiler.record_sample(sample);
        }
        
        let hotspots = profiler.top_hotspots(2);
        assert_eq!(hotspots.len(), 2);
        assert!(hotspots[0].1 >= hotspots[1].1); // First should have more or equal samples
    }
}
