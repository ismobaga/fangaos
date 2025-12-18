//! Power Management
//!
//! This module provides comprehensive power management functionality including:
//! - CPU frequency scaling and idle states (P-states, C-states)
//! - Device power state management (D0-D3)
//! - System suspend and resume (S1, S3)
//! - Hibernate support (S4)
//! - Battery monitoring and management
//!
//! # Overview
//!
//! The power management subsystem follows ACPI specifications for power states:
//!
//! ## CPU States
//! - **P-states**: Performance states controlling CPU frequency and voltage
//! - **C-states**: Idle states for power saving when CPU is not active
//!
//! ## Device States (D-states)
//! - **D0**: Fully functional
//! - **D1**: Low power with some functions available
//! - **D2**: Lower power with minimal functions
//! - **D3**: Off, context may be lost
//!
//! ## System Sleep States (S-states)
//! - **S0**: Working state
//! - **S1**: Standby - CPU stopped, RAM powered
//! - **S3**: Suspend to RAM - CPU off, RAM powered
//! - **S4**: Hibernate - System off, RAM saved to disk
//! - **S5**: Soft off
//!
//! # Usage
//!
//! ```no_run
//! use fanga_kernel::power;
//!
//! // Initialize power management
//! power::init();
//!
//! // Set CPU frequency scaling policy
//! power::cpu::set_scaling_policy(power::cpu::ScalingPolicy::PowerSave).unwrap();
//!
//! // Suspend system to RAM
//! power::suspend::suspend_to_ram().unwrap();
//!
//! // Check battery status
//! if power::battery::is_battery_low() {
//!     // Handle low battery
//! }
//! ```

pub mod cpu;
pub mod device;
pub mod suspend;
pub mod hibernate;
pub mod battery;

// Re-export commonly used types
pub use cpu::{PState, CState, ScalingPolicy, CpuPowerState};
pub use device::{DevicePowerState, DevicePowerCapabilities};
pub use suspend::SleepState;
pub use hibernate::HibernateState;
pub use battery::{BatteryStatus, PowerSource, BatteryInfo};

/// Initialize power management subsystem
pub fn init() {
    cpu::init();
    device::init();
    suspend::init();
    hibernate::init();
    battery::init();
}

/// Get system power summary
pub fn get_power_summary() -> PowerSummary {
    PowerSummary {
        cpu: cpu::get_power_state(),
        sleep_state: suspend::get_sleep_state(),
        hibernate_state: hibernate::get_state(),
        battery: battery::get_info(),
    }
}

/// System power summary
#[derive(Debug, Clone, Copy)]
pub struct PowerSummary {
    /// CPU power state
    pub cpu: CpuPowerState,
    /// System sleep state
    pub sleep_state: SleepState,
    /// Hibernate state
    pub hibernate_state: HibernateState,
    /// Battery information
    pub battery: BatteryInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_init() {
        init();
        
        let summary = get_power_summary();
        assert_eq!(summary.cpu.p_state, PState::P0);
        assert_eq!(summary.cpu.c_state, CState::C0);
        assert_eq!(summary.sleep_state, SleepState::S0);
        assert_eq!(summary.hibernate_state, HibernateState::Active);
    }

    #[test]
    fn test_power_subsystems() {
        init();
        
        // Test CPU power management
        cpu::set_scaling_policy(ScalingPolicy::PowerSave).unwrap();
        assert_eq!(cpu::get_scaling_policy(), ScalingPolicy::PowerSave);
        
        // Test device power management
        let caps = device::DevicePowerCapabilities::default();
        device::register_device("test".into(), caps, false).unwrap();
        device::set_device_state("test", DevicePowerState::D3).unwrap();
        assert_eq!(device::get_device_state("test").unwrap(), DevicePowerState::D3);
        device::unregister_device("test").unwrap();
        
        // Test suspend/resume
        assert_eq!(suspend::get_sleep_state(), SleepState::S0);
        
        // Test hibernate
        assert_eq!(hibernate::get_state(), HibernateState::Active);
        
        // Test battery
        assert!(!battery::is_battery_critical());
    }
}
