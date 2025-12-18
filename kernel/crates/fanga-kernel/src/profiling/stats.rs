//! Profiling Statistics

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Function-level statistics
#[derive(Debug, Clone)]
pub struct FunctionStats {
    /// Function name (or address if unknown)
    pub name: String,
    
    /// Number of samples in this function
    pub sample_count: usize,
    
    /// Percentage of total samples
    pub percentage: f32,
    
    /// Start address
    pub start_addr: u64,
    
    /// End address (if known)
    pub end_addr: Option<u64>,
}

impl FunctionStats {
    /// Create new function statistics
    pub fn new(name: String, sample_count: usize, total_samples: usize, start_addr: u64) -> Self {
        let percentage = if total_samples > 0 {
            (sample_count as f32 / total_samples as f32) * 100.0
        } else {
            0.0
        };
        
        Self {
            name,
            sample_count,
            percentage,
            start_addr,
            end_addr: None,
        }
    }
}

/// Profile statistics
#[derive(Debug, Clone)]
pub struct ProfileStats {
    /// Total number of samples
    pub total_samples: usize,
    
    /// Number of kernel samples
    pub kernel_samples: usize,
    
    /// Number of user samples
    pub user_samples: usize,
    
    /// Per-function statistics
    pub functions: Vec<FunctionStats>,
    
    /// Per-CPU sample counts
    pub cpu_samples: BTreeMap<usize, usize>,
}

impl ProfileStats {
    /// Create empty statistics
    pub fn new() -> Self {
        Self {
            total_samples: 0,
            kernel_samples: 0,
            user_samples: 0,
            functions: Vec::new(),
            cpu_samples: BTreeMap::new(),
        }
    }
    
    /// Create statistics from profiler data
    pub fn from_profiler(profiler: &super::sampler::Profiler) -> Self {
        let samples = profiler.samples();
        let sample_counts = profiler.sample_counts();
        
        let mut stats = Self::new();
        stats.total_samples = profiler.sample_count();
        
        // Count kernel vs user samples
        for sample in samples {
            if sample.kernel_mode {
                stats.kernel_samples += 1;
            } else {
                stats.user_samples += 1;
            }
            
            // Count per-CPU samples
            *stats.cpu_samples.entry(sample.cpu_id).or_insert(0) += 1;
        }
        
        // Build per-function statistics
        // For now, we treat each unique RIP as a separate "function"
        for (&rip, &count) in sample_counts {
            let name = format!("0x{:016x}", rip);
            let func_stats = FunctionStats::new(name, count, stats.total_samples, rip);
            stats.functions.push(func_stats);
        }
        
        // Sort by sample count
        stats.functions.sort_by(|a, b| b.sample_count.cmp(&a.sample_count));
        
        stats
    }
    
    /// Get kernel sample percentage
    pub fn kernel_percentage(&self) -> f32 {
        if self.total_samples > 0 {
            (self.kernel_samples as f32 / self.total_samples as f32) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get user sample percentage
    pub fn user_percentage(&self) -> f32 {
        if self.total_samples > 0 {
            (self.user_samples as f32 / self.total_samples as f32) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get top N functions by sample count
    pub fn top_functions(&self, n: usize) -> &[FunctionStats] {
        let end = core::cmp::min(n, self.functions.len());
        &self.functions[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_stats() {
        let stats = FunctionStats::new(
            String::from("test_func"),
            50,
            100,
            0x1000
        );
        
        assert_eq!(stats.name, "test_func");
        assert_eq!(stats.sample_count, 50);
        assert_eq!(stats.percentage, 50.0);
    }
    
    #[test]
    fn test_profile_stats_creation() {
        let stats = ProfileStats::new();
        assert_eq!(stats.total_samples, 0);
        assert_eq!(stats.kernel_samples, 0);
        assert_eq!(stats.user_samples, 0);
    }
    
    #[test]
    fn test_profile_stats_percentages() {
        let mut stats = ProfileStats::new();
        stats.total_samples = 100;
        stats.kernel_samples = 60;
        stats.user_samples = 40;
        
        // Use approximate comparison for floating point
        assert!((stats.kernel_percentage() - 60.0).abs() < 0.01);
        assert!((stats.user_percentage() - 40.0).abs() < 0.01);
    }
}
