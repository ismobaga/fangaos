//! Battery Management
//!
//! This module provides battery monitoring and management functionality including:
//! - Battery status monitoring
//! - Charge level tracking
//! - Power source detection

use spin::Mutex;

/// Battery status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    /// Battery is charging
    Charging,
    /// Battery is discharging
    Discharging,
    /// Battery is full
    Full,
    /// Battery not present
    NotPresent,
    /// Battery status unknown
    Unknown,
}

/// Power source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSource {
    /// AC power (wall outlet)
    AC,
    /// Battery power
    Battery,
    /// Unknown power source
    Unknown,
}

/// Battery information
#[derive(Debug, Clone, Copy)]
pub struct BatteryInfo {
    /// Battery status
    pub status: BatteryStatus,
    /// Charge percentage (0-100)
    pub charge_percent: u8,
    /// Is battery present
    pub is_present: bool,
    /// Remaining capacity in mWh
    pub capacity_mwh: u32,
    /// Design capacity in mWh
    pub design_capacity_mwh: u32,
    /// Current power source
    pub power_source: PowerSource,
    /// Estimated remaining time in minutes (0 if unknown)
    pub time_remaining_min: u32,
}

impl Default for BatteryInfo {
    fn default() -> Self {
        Self {
            status: BatteryStatus::Unknown,
            charge_percent: 0,
            is_present: false,
            capacity_mwh: 0,
            design_capacity_mwh: 0,
            power_source: PowerSource::Unknown,
            time_remaining_min: 0,
        }
    }
}

/// Global battery manager
static BATTERY: Mutex<BatteryInfo> = Mutex::new(BatteryInfo {
    status: BatteryStatus::Unknown,
    charge_percent: 0,
    is_present: false,
    capacity_mwh: 0,
    design_capacity_mwh: 0,
    power_source: PowerSource::Unknown,
    time_remaining_min: 0,
});

/// Initialize battery management
pub fn init() {
    let mut battery = BATTERY.lock();
    *battery = BatteryInfo::default();
    
    // In a real implementation:
    // 1. Detect battery presence via ACPI
    // 2. Read battery design capacity
    // 3. Set up battery status notifications
    
    // Mock initialization - assume no battery (desktop system)
    battery.is_present = false;
    battery.power_source = PowerSource::AC;
}

/// Update battery status
pub fn update() -> Result<(), &'static str> {
    // In a real implementation:
    // 1. Read ACPI battery status registers
    // 2. Update charge level
    // 3. Calculate time remaining
    // 4. Detect power source changes
    
    let mut battery = BATTERY.lock();
    
    if !battery.is_present {
        return Ok(());
    }
    
    // Mock update - simulate discharging
    if battery.status == BatteryStatus::Discharging && battery.charge_percent > 0 {
        battery.charge_percent = battery.charge_percent.saturating_sub(1);
        
        if battery.charge_percent == 0 {
            battery.status = BatteryStatus::NotPresent;
        }
    }
    
    Ok(())
}

/// Get battery information
pub fn get_info() -> BatteryInfo {
    *BATTERY.lock()
}

/// Get battery charge percentage
pub fn get_charge_percent() -> u8 {
    BATTERY.lock().charge_percent
}

/// Get battery status
pub fn get_status() -> BatteryStatus {
    BATTERY.lock().status
}

/// Get power source
pub fn get_power_source() -> PowerSource {
    BATTERY.lock().power_source
}

/// Check if battery is present
pub fn is_battery_present() -> bool {
    BATTERY.lock().is_present
}

/// Check if system is on AC power
pub fn is_on_ac_power() -> bool {
    BATTERY.lock().power_source == PowerSource::AC
}

/// Check if battery is charging
pub fn is_charging() -> bool {
    BATTERY.lock().status == BatteryStatus::Charging
}

/// Check if battery is low (below 20%)
pub fn is_battery_low() -> bool {
    let battery = BATTERY.lock();
    battery.is_present && battery.charge_percent < 20
}

/// Check if battery is critical (below 5%)
pub fn is_battery_critical() -> bool {
    let battery = BATTERY.lock();
    battery.is_present && battery.charge_percent < 5
}

/// Get estimated time remaining in minutes
pub fn get_time_remaining() -> u32 {
    BATTERY.lock().time_remaining_min
}

/// Set battery present (for testing)
#[cfg(test)]
pub fn set_battery_present(present: bool, capacity: u32, charge: u8) {
    let mut battery = BATTERY.lock();
    battery.is_present = present;
    battery.design_capacity_mwh = capacity;
    battery.capacity_mwh = (capacity * charge as u32) / 100;
    battery.charge_percent = charge;
    battery.status = if charge == 100 {
        BatteryStatus::Full
    } else {
        BatteryStatus::Discharging
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_init() {
        init();
        let info = get_info();
        assert!(!info.is_present);
        assert_eq!(info.power_source, PowerSource::AC);
    }

    #[test]
    fn test_battery_status() {
        init();
        
        // Simulate battery present
        set_battery_present(true, 50000, 80);
        
        assert!(is_battery_present());
        assert_eq!(get_charge_percent(), 80);
        assert!(!is_battery_low());
        assert!(!is_battery_critical());
    }

    #[test]
    fn test_battery_low_critical() {
        init();
        
        // Test low battery
        set_battery_present(true, 50000, 15);
        assert!(is_battery_low());
        assert!(!is_battery_critical());
        
        // Test critical battery
        set_battery_present(true, 50000, 3);
        assert!(is_battery_low());
        assert!(is_battery_critical());
    }

    #[test]
    fn test_battery_full() {
        init();
        
        set_battery_present(true, 50000, 100);
        assert_eq!(get_status(), BatteryStatus::Full);
        assert_eq!(get_charge_percent(), 100);
    }

    #[test]
    fn test_ac_power() {
        init();
        
        assert!(is_on_ac_power());
        assert!(!is_charging());
    }

    #[test]
    fn test_battery_update() {
        init();
        
        set_battery_present(true, 50000, 50);
        
        // Update should work
        update().unwrap();
        
        // Charge should decrease (mock implementation)
        assert!(get_charge_percent() <= 50);
    }
}
