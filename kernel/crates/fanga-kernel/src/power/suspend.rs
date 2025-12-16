//! Suspend/Resume Support
//!
//! This module provides system suspend and resume functionality including:
//! - Sleep states (S1-S5)
//! - Suspend to RAM (S3)
//! - Suspend preparation and restoration

use spin::Mutex;

/// System Sleep State (ACPI S-states)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepState {
    /// Working state
    S0,
    /// Standby/Sleep - CPU stopped, RAM powered
    S1,
    /// Suspend to RAM - CPU off, RAM powered
    S3,
    /// Hibernate - System off, RAM to disk
    S4,
    /// Soft off - System off
    S5,
}

/// System power state
#[derive(Debug, Clone, Copy)]
pub struct SystemPowerState {
    /// Current sleep state
    pub sleep_state: SleepState,
    /// System is suspending
    pub is_suspending: bool,
    /// System is resuming
    pub is_resuming: bool,
}

impl Default for SystemPowerState {
    fn default() -> Self {
        Self {
            sleep_state: SleepState::S0,
            is_suspending: false,
            is_resuming: false,
        }
    }
}

/// Global system power state
static SYSTEM_POWER: Mutex<SystemPowerState> = Mutex::new(SystemPowerState {
    sleep_state: SleepState::S0,
    is_suspending: false,
    is_resuming: false,
});

/// Initialize suspend/resume subsystem
pub fn init() {
    let mut state = SYSTEM_POWER.lock();
    state.sleep_state = SleepState::S0;
    state.is_suspending = false;
    state.is_resuming = false;
}

/// Prepare system for suspend
fn prepare_suspend() -> Result<(), &'static str> {
    // In a real implementation:
    // 1. Notify all drivers to save state
    // 2. Flush disk caches
    // 3. Stop non-essential processes
    // 4. Save CPU context
    
    // For now, just suspend devices
    super::device::suspend_all_devices()?;
    
    Ok(())
}

/// Restore system after resume
fn restore_resume() -> Result<(), &'static str> {
    // In a real implementation:
    // 1. Restore CPU context
    // 2. Resume all drivers
    // 3. Restore device states
    // 4. Resume processes
    
    // For now, just resume devices
    super::device::resume_all_devices()?;
    
    Ok(())
}

/// Suspend system to RAM (S3 state)
pub fn suspend_to_ram() -> Result<(), &'static str> {
    let mut state = SYSTEM_POWER.lock();
    
    if state.sleep_state != SleepState::S0 {
        return Err("System not in S0 state");
    }
    
    state.is_suspending = true;
    drop(state);
    
    // Prepare for suspend
    prepare_suspend()?;
    
    // Enter suspend state
    let mut state = SYSTEM_POWER.lock();
    state.sleep_state = SleepState::S3;
    state.is_suspending = false;
    
    // In a real implementation, we would:
    // 1. Write ACPI PM1 control register to enter S3
    // 2. CPU would halt and system would power down
    // 3. On wake event (keyboard, network, timer), firmware would resume
    // 4. Control would return here
    
    Ok(())
}

/// Resume system from suspend
pub fn resume_from_suspend() -> Result<(), &'static str> {
    let mut state = SYSTEM_POWER.lock();
    
    if state.sleep_state == SleepState::S0 {
        return Err("System not suspended");
    }
    
    state.is_resuming = true;
    let prev_state = state.sleep_state;
    drop(state);
    
    // Restore system state
    restore_resume()?;
    
    let mut state = SYSTEM_POWER.lock();
    state.sleep_state = SleepState::S0;
    state.is_resuming = false;
    
    Ok(())
}

/// Check if system is suspending
pub fn is_suspending() -> bool {
    SYSTEM_POWER.lock().is_suspending
}

/// Check if system is resuming
pub fn is_resuming() -> bool {
    SYSTEM_POWER.lock().is_resuming
}

/// Get current sleep state
pub fn get_sleep_state() -> SleepState {
    SYSTEM_POWER.lock().sleep_state
}

/// Standby (S1 state) - quick suspend
pub fn standby() -> Result<(), &'static str> {
    let mut state = SYSTEM_POWER.lock();
    
    if state.sleep_state != SleepState::S0 {
        return Err("System not in S0 state");
    }
    
    // S1 is a lighter suspend, just stop CPU
    state.sleep_state = SleepState::S1;
    drop(state);
    
    // Halt CPU
    super::cpu::enter_c_state(super::cpu::CState::C1)?;
    
    // When we wake up, restore S0 state
    let mut state = SYSTEM_POWER.lock();
    state.sleep_state = SleepState::S0;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspend_init() {
        init();
        assert_eq!(get_sleep_state(), SleepState::S0);
        assert!(!is_suspending());
        assert!(!is_resuming());
    }

    #[test]
    fn test_suspend_to_ram() {
        init();
        super::super::device::init();
        
        // Test suspend
        suspend_to_ram().unwrap();
        assert_eq!(get_sleep_state(), SleepState::S3);
        
        // Test resume
        resume_from_suspend().unwrap();
        assert_eq!(get_sleep_state(), SleepState::S0);
    }

    #[test]
    fn test_standby() {
        init();
        
        // Standby should work
        // Note: In test, halt is a no-op
        standby().unwrap();
        assert_eq!(get_sleep_state(), SleepState::S0);
    }

    #[test]
    fn test_invalid_suspend() {
        init();
        super::super::device::init();
        
        // Suspend once
        suspend_to_ram().unwrap();
        
        // Try to suspend again - should fail
        let result = suspend_to_ram();
        assert!(result.is_err());
        
        // Resume
        resume_from_suspend().unwrap();
    }
}
