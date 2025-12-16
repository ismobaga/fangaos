//! CPU Power Management
//!
//! This module provides CPU power management functionality including:
//! - CPU frequency scaling (P-states)
//! - CPU idle states (C-states)
//! - CPU power state transitions

use spin::Mutex;

/// CPU Performance State (P-state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PState {
    /// Maximum performance, highest frequency
    P0,
    /// Medium performance
    P1,
    /// Low performance, power saving
    P2,
    /// Minimum performance, maximum power saving
    P3,
}

/// CPU Idle State (C-state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CState {
    /// CPU running, no power saving
    C0,
    /// Halt state, clock gating
    C1,
    /// Stop-clock state
    C2,
    /// Deep sleep, cache flushed
    C3,
}

/// CPU frequency scaling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingPolicy {
    /// Maximum performance
    Performance,
    /// Balance performance and power
    Balanced,
    /// Maximum power saving
    PowerSave,
}

/// CPU power management state
#[derive(Debug, Clone, Copy)]
pub struct CpuPowerState {
    /// Current P-state
    pub p_state: PState,
    /// Current C-state
    pub c_state: CState,
    /// Current scaling policy
    pub policy: ScalingPolicy,
    /// CPU frequency in MHz (estimated)
    pub frequency_mhz: u32,
}

impl Default for CpuPowerState {
    fn default() -> Self {
        Self {
            p_state: PState::P0,
            c_state: CState::C0,
            policy: ScalingPolicy::Balanced,
            frequency_mhz: 0,
        }
    }
}

/// Global CPU power manager
static CPU_POWER: Mutex<CpuPowerState> = Mutex::new(CpuPowerState {
    p_state: PState::P0,
    c_state: CState::C0,
    policy: ScalingPolicy::Balanced,
    frequency_mhz: 0,
});

/// Initialize CPU power management
pub fn init() {
    let mut state = CPU_POWER.lock();
    state.p_state = PState::P0;
    state.c_state = CState::C0;
    state.policy = ScalingPolicy::Balanced;
    
    // Estimate base frequency (would require CPUID or MSR reads in real implementation)
    state.frequency_mhz = 2400; // Default estimate
}

/// Set CPU frequency scaling policy
pub fn set_scaling_policy(policy: ScalingPolicy) -> Result<(), &'static str> {
    let mut state = CPU_POWER.lock();
    state.policy = policy;
    
    // Adjust P-state based on policy
    match policy {
        ScalingPolicy::Performance => {
            state.p_state = PState::P0;
            state.frequency_mhz = 2400;
        }
        ScalingPolicy::Balanced => {
            state.p_state = PState::P1;
            state.frequency_mhz = 2000;
        }
        ScalingPolicy::PowerSave => {
            state.p_state = PState::P2;
            state.frequency_mhz = 1600;
        }
    }
    
    Ok(())
}

/// Set CPU P-state (performance state)
pub fn set_p_state(p_state: PState) -> Result<(), &'static str> {
    let mut state = CPU_POWER.lock();
    state.p_state = p_state;
    
    // Update frequency based on P-state
    state.frequency_mhz = match p_state {
        PState::P0 => 2400,
        PState::P1 => 2000,
        PState::P2 => 1600,
        PState::P3 => 1200,
    };
    
    Ok(())
}

/// Get current CPU P-state
pub fn get_p_state() -> PState {
    CPU_POWER.lock().p_state
}

/// Enter CPU idle state (C-state)
pub fn enter_c_state(c_state: CState) -> Result<(), &'static str> {
    let mut state = CPU_POWER.lock();
    
    match c_state {
        CState::C0 => {
            // Normal running state, no action needed
            state.c_state = CState::C0;
        }
        CState::C1 => {
            // Enter halt state
            state.c_state = CState::C1;
            drop(state); // Release lock before halting
            cpu_halt();
        }
        CState::C2 | CState::C3 => {
            // Deeper sleep states (not fully implemented)
            state.c_state = c_state;
            drop(state);
            cpu_halt();
        }
    }
    
    Ok(())
}

/// Get current CPU C-state
pub fn get_c_state() -> CState {
    CPU_POWER.lock().c_state
}

/// Get current CPU frequency in MHz
pub fn get_frequency_mhz() -> u32 {
    CPU_POWER.lock().frequency_mhz
}

/// Get current scaling policy
pub fn get_scaling_policy() -> ScalingPolicy {
    CPU_POWER.lock().policy
}

/// Halt the CPU until next interrupt (C1 state)
#[cfg(not(test))]
fn cpu_halt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}

#[cfg(test)]
fn cpu_halt() {
    // Do nothing in tests
}

/// Get full CPU power state
pub fn get_power_state() -> CpuPowerState {
    *CPU_POWER.lock()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_power_init() {
        init();
        let state = get_power_state();
        assert_eq!(state.p_state, PState::P0);
        assert_eq!(state.c_state, CState::C0);
    }

    #[test]
    fn test_set_scaling_policy() {
        set_scaling_policy(ScalingPolicy::PowerSave).unwrap();
        assert_eq!(get_scaling_policy(), ScalingPolicy::PowerSave);
        assert_eq!(get_p_state(), PState::P2);
        
        set_scaling_policy(ScalingPolicy::Performance).unwrap();
        assert_eq!(get_scaling_policy(), ScalingPolicy::Performance);
        assert_eq!(get_p_state(), PState::P0);
    }

    #[test]
    fn test_set_p_state() {
        set_p_state(PState::P3).unwrap();
        assert_eq!(get_p_state(), PState::P3);
        assert_eq!(get_frequency_mhz(), 1200);
        
        set_p_state(PState::P0).unwrap();
        assert_eq!(get_p_state(), PState::P0);
        assert_eq!(get_frequency_mhz(), 2400);
    }

    #[test]
    fn test_c_state() {
        enter_c_state(CState::C0).unwrap();
        assert_eq!(get_c_state(), CState::C0);
        
        // C1 would halt CPU, so we just test the state change part
        // In real hardware, interrupt would wake it up
    }
}
