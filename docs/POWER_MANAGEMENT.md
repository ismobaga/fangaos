# Power Management

FangaOS includes a comprehensive power management subsystem that provides ACPI-style power control for CPUs, devices, and the system as a whole.

## Overview

The power management subsystem implements multiple layers of power control:

1. **CPU Power Management** - Control CPU frequency and idle states
2. **Device Power Management** - Manage individual device power states
3. **System Sleep States** - Suspend and resume the entire system
4. **Hibernation** - Save system state to disk and power off
5. **Battery Management** - Monitor battery status and power source

## Architecture

```
┌─────────────────────────────────────────────────┐
│           Power Management API                   │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │   CPU    │  │  Device  │  │ Suspend  │     │
│  │  Power   │  │  Power   │  │  Resume  │     │
│  └──────────┘  └──────────┘  └──────────┘     │
│                                                  │
│  ┌──────────┐  ┌──────────┐                    │
│  │Hibernate │  │ Battery  │                    │
│  │ Support  │  │  Manager │                    │
│  └──────────┘  └──────────┘                    │
│                                                  │
└─────────────────────────────────────────────────┘
```

## CPU Power Management

### P-states (Performance States)

P-states control CPU frequency and voltage to balance performance with power consumption:

- **P0**: Maximum performance (highest frequency)
- **P1**: Medium performance
- **P2**: Low performance
- **P3**: Minimum performance (maximum power saving)

```rust
use fanga_kernel::power;

// Set P-state manually
power::cpu::set_p_state(power::PState::P2).unwrap();

// Or use a scaling policy
power::cpu::set_scaling_policy(power::ScalingPolicy::PowerSave).unwrap();
```

### C-states (Idle States)

C-states control power consumption when the CPU is idle:

- **C0**: CPU running (no power saving)
- **C1**: Halt state (clock gating)
- **C2**: Stop-clock state
- **C3**: Deep sleep (cache flushed)

```rust
// Enter low-power idle state
power::cpu::enter_c_state(power::CState::C1).unwrap();
```

### Scaling Policies

Three pre-defined scaling policies are available:

1. **Performance**: Maximum CPU frequency (P0 state)
2. **Balanced**: Medium CPU frequency (P1 state)
3. **PowerSave**: Minimum CPU frequency (P2 state)

```rust
// Set policy to power save mode
power::cpu::set_scaling_policy(power::ScalingPolicy::PowerSave).unwrap();

// Check current policy
let policy = power::cpu::get_scaling_policy();
```

## Device Power Management

### D-states (Device Power States)

Devices support four power states following ACPI specifications:

- **D0**: Fully on and functional
- **D1**: Low power with some functions available
- **D2**: Lower power with minimal functions
- **D3**: Off (context may be lost)

### Device Registration

Devices must be registered with the power manager before power control:

```rust
use fanga_kernel::power;
use alloc::string::String;

// Define device capabilities
let caps = power::DevicePowerCapabilities {
    supports_d1: true,
    supports_d2: true,
    supports_d3: true,
    wake_from_d1: false,
    wake_from_d2: false,
    wake_from_d3: true,  // Device can wake from D3
};

// Register device
power::device::register_device(
    String::from("disk0"),
    caps,
    false  // not critical
).unwrap();
```

### Device Power Control

```rust
// Power down a device
power::device::set_device_state("disk0", power::DevicePowerState::D3).unwrap();

// Power up a device
power::device::set_device_state("disk0", power::DevicePowerState::D0).unwrap();

// Check device state
let state = power::device::get_device_state("disk0").unwrap();
```

### Critical Devices

Critical devices cannot be powered down to preserve system stability:

```rust
// Register a critical device
power::device::register_device(
    String::from("system_controller"),
    caps,
    true  // critical
).unwrap();

// This will fail - cannot power down critical devices
power::device::set_device_state("system_controller", power::DevicePowerState::D3).unwrap_err();
```

## System Sleep States

### S-states (System Sleep States)

Following ACPI specification:

- **S0**: Working state
- **S1**: Standby - CPU stopped, RAM powered
- **S3**: Suspend to RAM - CPU off, RAM powered
- **S4**: Hibernate - System off, RAM saved to disk
- **S5**: Soft off

### Suspend to RAM (S3)

Suspend saves the system state in RAM and powers down most components:

```rust
use fanga_kernel::power;

// Suspend system
power::suspend::suspend_to_ram().unwrap();

// System will resume here when woken up
power::suspend::resume_from_suspend().unwrap();
```

### Standby (S1)

Quick suspend mode that only stops the CPU:

```rust
// Enter standby mode
power::suspend::standby().unwrap();
// System continues here after wake
```

### Suspend Workflow

1. **Preparation**:
   - Notify all drivers to save state
   - Flush disk caches
   - Stop non-essential processes
   - Power down non-critical devices

2. **Suspend**:
   - Save CPU context
   - Enter sleep state

3. **Resume**:
   - Restore CPU context
   - Resume all drivers
   - Restore device states
   - Resume processes

## Hibernation (S4)

Hibernation saves the entire system state to disk and powers off completely:

```rust
use fanga_kernel::power;

// Enable hibernation
power::hibernate::enable().unwrap();

// Hibernate system
power::hibernate::hibernate().unwrap();

// On next boot, restore from hibernate
if power::hibernate::has_valid_image() {
    power::hibernate::restore_from_hibernate().unwrap();
}
```

### Hibernate Process

1. **Save State**:
   - Suspend all devices
   - Create memory snapshot
   - Compress and write to disk

2. **Power Off**:
   - System powers down completely

3. **Restore** (on boot):
   - Detect hibernate image
   - Read and decompress image
   - Restore memory pages
   - Restore CPU context
   - Resume devices

## Battery Management

### Battery Status

Monitor battery level and power source:

```rust
use fanga_kernel::power;

// Get battery information
let battery = power::battery::get_info();

println!("Battery present: {}", battery.is_present);
println!("Charge: {}%", battery.charge_percent);
println!("Status: {:?}", battery.status);

// Check specific conditions
if power::battery::is_battery_low() {
    println!("Warning: Battery low!");
}

if power::battery::is_battery_critical() {
    println!("Critical: Battery very low!");
}

// Check power source
if power::battery::is_on_ac_power() {
    println!("Running on AC power");
} else {
    println!("Running on battery");
}
```

### Battery States

- **Charging**: Battery is charging from AC power
- **Discharging**: Running on battery power
- **Full**: Battery fully charged
- **NotPresent**: No battery (desktop system)
- **Unknown**: Battery status unknown

## Shell Commands

The FangaOS shell provides a `power` command for power management:

### Display Power Status

```bash
fangaos> power
Power Management Status:

CPU:
  P-state: P1 (Medium Performance)
  C-state: C0 (Running)
  Policy: Balanced
  Frequency: 2000 MHz

System:
  Sleep State: S0 (Working)

Battery:
  Not Present (AC Power)
```

### Set CPU Scaling Policy

```bash
fangaos> power policy performance    # Maximum performance
fangaos> power policy balanced       # Balance power and performance
fangaos> power policy powersave      # Maximum power saving
```

### List Power-Managed Devices

```bash
fangaos> power devices
Registered Power-Managed Devices:
  - disk0: D0 (On)
  - network0: D0 (On)
  - usb_controller: D0 (On)
```

### Check System Status

```bash
fangaos> power status
Power Status: Active (S0)
```

## Implementation Details

### Module Structure

```
kernel/crates/fanga-kernel/src/power/
├── mod.rs         # Main module with public API
├── cpu.rs         # CPU power management (P-states, C-states)
├── device.rs      # Device power state management
├── suspend.rs     # Suspend/resume functionality
├── hibernate.rs   # Hibernation support
└── battery.rs     # Battery monitoring
```

### Initialization

Power management is initialized during kernel boot:

```rust
// In kernel/crates/fanga-kernel/src/main.rs
power::init();
```

This initializes all subsystems:
- CPU power management
- Device power management
- Suspend/resume system
- Hibernation support
- Battery management

### Thread Safety

All power management operations are thread-safe using Rust's `Mutex`:

```rust
static CPU_POWER: Mutex<CpuPowerState> = Mutex::new(/* ... */);
static DEVICE_POWER: Mutex<BTreeMap<String, DeviceInfo>> = Mutex::new(/* ... */);
```

## Testing

The power management subsystem includes comprehensive unit tests:

```bash
# Run power management tests
cd kernel/crates/fanga-kernel
cargo test --lib power
```

Test coverage includes:
- CPU P-state and C-state transitions
- Device registration and power control
- Suspend/resume cycles
- Hibernation enable/disable and cycles
- Battery status monitoring
- Critical device protection

## Future Enhancements

Planned improvements for the power management subsystem:

1. **ACPI Support**:
   - Parse ACPI tables for hardware capabilities
   - Use ACPI methods for power control
   - Support ACPI events and notifications

2. **Advanced CPU Features**:
   - Read actual CPU frequency via CPUID
   - Control frequency via MSR writes
   - Support Intel SpeedStep / AMD PowerNow
   - Per-core power management

3. **Enhanced Device Management**:
   - Runtime power management
   - Autosuspend for idle devices
   - Wake-on-LAN and other wake sources
   - Device power domains

4. **Thermal Management**:
   - Temperature monitoring
   - Thermal throttling
   - Fan control
   - Thermal zones

5. **Power Profiles**:
   - User-defined power profiles
   - Automatic profile switching
   - Application-specific policies

## References

- [ACPI Specification](https://uefi.org/specifications)
- [Intel Software Developer Manuals](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
- [AMD Processor Programming Reference](https://developer.amd.com/resources/developer-guides-manuals/)
- [OSDev Wiki: Power Management](https://wiki.osdev.org/ACPI)

## Examples

### Example 1: Power-Aware Application

```rust
use fanga_kernel::power;

fn main() {
    // Check if on battery
    if power::battery::is_battery_present() {
        // Reduce CPU frequency to save power
        power::cpu::set_scaling_policy(power::ScalingPolicy::PowerSave).unwrap();
        
        // Power down unused devices
        power::device::set_device_state("unused_device", power::DevicePowerState::D3).unwrap();
    } else {
        // On AC power, use maximum performance
        power::cpu::set_scaling_policy(power::ScalingPolicy::Performance).unwrap();
    }
    
    // Do work...
    
    // Return to balanced mode
    power::cpu::set_scaling_policy(power::ScalingPolicy::Balanced).unwrap();
}
```

### Example 2: System Suspend

```rust
use fanga_kernel::power;

fn idle_timeout_handler() {
    // System has been idle for 10 minutes
    println!("System idle, suspending...");
    
    // Suspend to RAM
    if let Err(e) = power::suspend::suspend_to_ram() {
        println!("Suspend failed: {}", e);
        return;
    }
    
    // Execution continues here after resume
    println!("System resumed!");
}
```

### Example 3: Low Battery Handler

```rust
use fanga_kernel::power;

fn battery_monitor() {
    loop {
        power::battery::update().unwrap();
        
        if power::battery::is_battery_critical() {
            // Battery critical, hibernate immediately
            println!("Critical battery! Hibernating...");
            power::hibernate::hibernate().unwrap();
        } else if power::battery::is_battery_low() {
            // Battery low, switch to power save mode
            power::cpu::set_scaling_policy(power::ScalingPolicy::PowerSave).unwrap();
            println!("Low battery, switched to power save mode");
        }
        
        // Sleep for 60 seconds
        task::sleep_ms(60000);
    }
}
```

## Conclusion

The FangaOS power management subsystem provides comprehensive control over system power consumption while maintaining compatibility with ACPI specifications. The modular design allows for easy extension and integration with additional hardware capabilities as the OS evolves.
