//! Device Power Management
//!
//! This module provides device power state management including:
//! - Device power states (D0-D3)
//! - Device power state transitions
//! - Device power policy

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Device Power State (ACPI D-states)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePowerState {
    /// Fully on, fully functional
    D0,
    /// Low power, some functions available
    D1,
    /// Lower power, minimal functions
    D2,
    /// Off, context may be lost
    D3,
}

/// Device power capabilities
#[derive(Debug, Clone, Copy)]
pub struct DevicePowerCapabilities {
    /// Device supports D1 state
    pub supports_d1: bool,
    /// Device supports D2 state
    pub supports_d2: bool,
    /// Device supports D3 state
    pub supports_d3: bool,
    /// Device can wake from D1
    pub wake_from_d1: bool,
    /// Device can wake from D2
    pub wake_from_d2: bool,
    /// Device can wake from D3
    pub wake_from_d3: bool,
}

impl Default for DevicePowerCapabilities {
    fn default() -> Self {
        Self {
            supports_d1: true,
            supports_d2: true,
            supports_d3: true,
            wake_from_d1: false,
            wake_from_d2: false,
            wake_from_d3: false,
        }
    }
}

/// Device power information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Current power state
    pub state: DevicePowerState,
    /// Power capabilities
    pub capabilities: DevicePowerCapabilities,
    /// Device is critical (cannot be powered down)
    pub is_critical: bool,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            state: DevicePowerState::D0,
            capabilities: DevicePowerCapabilities::default(),
            is_critical: false,
        }
    }
}

/// Global device power manager
static DEVICE_POWER: Mutex<BTreeMap<String, DeviceInfo>> = Mutex::new(BTreeMap::new());

/// Initialize device power management
pub fn init() {
    let mut devices = DEVICE_POWER.lock();
    devices.clear();
}

/// Register a device for power management
pub fn register_device(
    name: String,
    capabilities: DevicePowerCapabilities,
    is_critical: bool,
) -> Result<(), &'static str> {
    let mut devices = DEVICE_POWER.lock();
    
    if devices.contains_key(&name) {
        return Err("Device already registered");
    }
    
    devices.insert(name, DeviceInfo {
        state: DevicePowerState::D0,
        capabilities,
        is_critical,
    });
    
    Ok(())
}

/// Unregister a device from power management
pub fn unregister_device(name: &str) -> Result<(), &'static str> {
    let mut devices = DEVICE_POWER.lock();
    
    if devices.remove(name).is_none() {
        return Err("Device not found");
    }
    
    Ok(())
}

/// Set device power state
pub fn set_device_state(name: &str, state: DevicePowerState) -> Result<(), &'static str> {
    let mut devices = DEVICE_POWER.lock();
    
    let device = devices.get_mut(name).ok_or("Device not found")?;
    
    // Check if device is critical
    if device.is_critical && state != DevicePowerState::D0 {
        return Err("Cannot power down critical device");
    }
    
    // Check if device supports the requested state
    match state {
        DevicePowerState::D0 => {
            // Always supported
        }
        DevicePowerState::D1 => {
            if !device.capabilities.supports_d1 {
                return Err("Device does not support D1 state");
            }
        }
        DevicePowerState::D2 => {
            if !device.capabilities.supports_d2 {
                return Err("Device does not support D2 state");
            }
        }
        DevicePowerState::D3 => {
            if !device.capabilities.supports_d3 {
                return Err("Device does not support D3 state");
            }
        }
    }
    
    device.state = state;
    Ok(())
}

/// Get device power state
pub fn get_device_state(name: &str) -> Result<DevicePowerState, &'static str> {
    let devices = DEVICE_POWER.lock();
    devices.get(name)
        .map(|d| d.state)
        .ok_or("Device not found")
}

/// Power down all non-critical devices
pub fn suspend_all_devices() -> Result<(), &'static str> {
    let mut devices = DEVICE_POWER.lock();
    
    for (name, device) in devices.iter_mut() {
        if !device.is_critical && device.capabilities.supports_d3 {
            device.state = DevicePowerState::D3;
        }
    }
    
    Ok(())
}

/// Resume all devices to D0 state
pub fn resume_all_devices() -> Result<(), &'static str> {
    let mut devices = DEVICE_POWER.lock();
    
    for (name, device) in devices.iter_mut() {
        device.state = DevicePowerState::D0;
    }
    
    Ok(())
}

/// Get list of registered devices
pub fn list_devices() -> alloc::vec::Vec<(String, DevicePowerState)> {
    let devices = DEVICE_POWER.lock();
    devices.iter()
        .map(|(name, info)| (name.clone(), info.state))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_registration() {
        init();
        
        let caps = DevicePowerCapabilities::default();
        register_device(String::from("test_device"), caps, false).unwrap();
        
        let state = get_device_state("test_device").unwrap();
        assert_eq!(state, DevicePowerState::D0);
        
        unregister_device("test_device").unwrap();
        assert!(get_device_state("test_device").is_err());
    }

    #[test]
    fn test_device_state_transitions() {
        init();
        
        let caps = DevicePowerCapabilities::default();
        register_device(String::from("disk"), caps, false).unwrap();
        
        set_device_state("disk", DevicePowerState::D3).unwrap();
        assert_eq!(get_device_state("disk").unwrap(), DevicePowerState::D3);
        
        set_device_state("disk", DevicePowerState::D0).unwrap();
        assert_eq!(get_device_state("disk").unwrap(), DevicePowerState::D0);
        
        unregister_device("disk").unwrap();
    }

    #[test]
    fn test_critical_device() {
        init();
        
        let caps = DevicePowerCapabilities::default();
        register_device(String::from("critical"), caps, true).unwrap();
        
        // Should not be able to power down critical device
        let result = set_device_state("critical", DevicePowerState::D3);
        assert!(result.is_err());
        
        assert_eq!(get_device_state("critical").unwrap(), DevicePowerState::D0);
        
        unregister_device("critical").unwrap();
    }

    #[test]
    fn test_suspend_resume_all() {
        init();
        
        let caps = DevicePowerCapabilities::default();
        register_device(String::from("dev1"), caps, false).unwrap();
        register_device(String::from("dev2"), caps, false).unwrap();
        register_device(String::from("critical"), caps, true).unwrap();
        
        suspend_all_devices().unwrap();
        
        // Non-critical devices should be in D3
        assert_eq!(get_device_state("dev1").unwrap(), DevicePowerState::D3);
        assert_eq!(get_device_state("dev2").unwrap(), DevicePowerState::D3);
        // Critical device should still be in D0
        assert_eq!(get_device_state("critical").unwrap(), DevicePowerState::D0);
        
        resume_all_devices().unwrap();
        
        // All devices should be in D0
        assert_eq!(get_device_state("dev1").unwrap(), DevicePowerState::D0);
        assert_eq!(get_device_state("dev2").unwrap(), DevicePowerState::D0);
        assert_eq!(get_device_state("critical").unwrap(), DevicePowerState::D0);
        
        unregister_device("dev1").unwrap();
        unregister_device("dev2").unwrap();
        unregister_device("critical").unwrap();
    }
}
