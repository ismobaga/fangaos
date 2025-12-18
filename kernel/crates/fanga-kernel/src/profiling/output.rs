//! Profiling Output and Export

use super::stats::ProfileStats;
extern crate alloc;
use alloc::string::String;
use alloc::format;

/// Output format for profile data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text format
    Text,
    
    /// JSON format
    Json,
    
    /// CSV format
    Csv,
}

/// Profile output generator
pub struct ProfileOutput {
    /// Statistics to output
    stats: ProfileStats,
    
    /// Output format
    format: OutputFormat,
}

impl ProfileOutput {
    /// Create a new output generator
    pub fn new(stats: ProfileStats, format: OutputFormat) -> Self {
        Self { stats, format }
    }
    
    /// Generate output string
    pub fn generate(&self) -> String {
        match self.format {
            OutputFormat::Text => self.generate_text(),
            OutputFormat::Json => self.generate_json(),
            OutputFormat::Csv => self.generate_csv(),
        }
    }
    
    /// Generate text output
    fn generate_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str("=== Performance Profile ===\n\n");
        output.push_str(&format!("Total samples: {}\n", self.stats.total_samples));
        output.push_str(&format!("Kernel samples: {} ({:.2}%)\n", 
            self.stats.kernel_samples, self.stats.kernel_percentage()));
        output.push_str(&format!("User samples: {} ({:.2}%)\n", 
            self.stats.user_samples, self.stats.user_percentage()));
        
        output.push_str("\n=== Top Functions ===\n\n");
        output.push_str("Samples  %      Function\n");
        output.push_str("-------  -----  --------\n");
        
        for func in self.stats.top_functions(20) {
            output.push_str(&format!("{:7}  {:5.2}  {}\n",
                func.sample_count,
                func.percentage,
                func.name
            ));
        }
        
        if !self.stats.cpu_samples.is_empty() {
            output.push_str("\n=== Per-CPU Distribution ===\n\n");
            for (cpu_id, count) in &self.stats.cpu_samples {
                let percentage = (*count as f32 / self.stats.total_samples as f32) * 100.0;
                output.push_str(&format!("CPU {}: {} samples ({:.2}%)\n",
                    cpu_id, count, percentage
                ));
            }
        }
        
        output
    }
    
    /// Generate JSON output
    fn generate_json(&self) -> String {
        let mut output = String::from("{\n");
        
        output.push_str(&format!("  \"total_samples\": {},\n", self.stats.total_samples));
        output.push_str(&format!("  \"kernel_samples\": {},\n", self.stats.kernel_samples));
        output.push_str(&format!("  \"user_samples\": {},\n", self.stats.user_samples));
        
        output.push_str("  \"functions\": [\n");
        let funcs = self.stats.top_functions(20);
        for (i, func) in funcs.iter().enumerate() {
            output.push_str(&format!("    {{\"name\": \"{}\", \"samples\": {}, \"percentage\": {:.2}}}",
                func.name, func.sample_count, func.percentage
            ));
            if i < funcs.len() - 1 {
                output.push_str(",");
            }
            output.push_str("\n");
        }
        output.push_str("  ]\n");
        
        output.push_str("}\n");
        output
    }
    
    /// Generate CSV output
    fn generate_csv(&self) -> String {
        let mut output = String::from("Function,Samples,Percentage\n");
        
        for func in self.stats.top_functions(20) {
            output.push_str(&format!("{},{},{:.2}\n",
                func.name,
                func.sample_count,
                func.percentage
            ));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    
    #[test]
    fn test_output_format() {
        let stats = ProfileStats::new();
        let output = ProfileOutput::new(stats, OutputFormat::Text);
        
        let text = output.generate();
        assert!(text.contains("Performance Profile"));
    }
    
    #[test]
    fn test_text_output() {
        let mut stats = ProfileStats::new();
        stats.total_samples = 100;
        stats.kernel_samples = 60;
        stats.user_samples = 40;
        
        let output = ProfileOutput::new(stats, OutputFormat::Text);
        let text = output.generate();
        
        assert!(text.contains("Total samples: 100"));
        assert!(text.contains("Kernel samples: 60"));
    }
    
    #[test]
    fn test_json_output() {
        let stats = ProfileStats::new();
        let output = ProfileOutput::new(stats, OutputFormat::Json);
        let json = output.generate();
        
        assert!(json.contains("\"total_samples\""));
        assert!(json.contains("\"functions\""));
    }
    
    #[test]
    fn test_csv_output() {
        let stats = ProfileStats::new();
        let output = ProfileOutput::new(stats, OutputFormat::Csv);
        let csv = output.generate();
        
        assert!(csv.contains("Function,Samples,Percentage"));
    }
}
