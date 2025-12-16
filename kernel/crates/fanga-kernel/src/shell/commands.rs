/// Built-in shell commands
///
/// Implements the core shell commands:
/// - help: Display available commands
/// - clear: Clear the screen
/// - echo: Echo arguments
/// - memory: Display memory statistics
/// - ps: Display process/task list
/// - exit: Exit/halt the system

use alloc::vec::Vec;
use crate::io::framebuffer;
use crate::memory;
use crate::task;
use crate::power;

/// Execute a command
pub fn execute(command: &str, args: Vec<&str>, shell: &mut super::Shell) -> Result<(), &'static str> {
    match command {
        "" => Ok(()), // Empty command, do nothing
        "help" => cmd_help(),
        "clear" => cmd_clear(),
        "echo" => cmd_echo(args),
        "memory" => cmd_memory(),
        "ps" => cmd_ps(),
        "power" => cmd_power(args),
        "exit" => cmd_exit(shell),
        _ => {
            let mut fb = framebuffer::framebuffer();
            fb.write_string("Unknown command: ");
            fb.write_string(command);
            fb.write_string("\n");
            fb.write_string("Type 'help' for available commands.\n");
            Ok(())
        }
    }
}

/// Display help information
fn cmd_help() -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    fb.write_string("FangaOS Shell - Available Commands:\n");
    fb.write_string("  help    - Display this help message\n");
    fb.write_string("  clear   - Clear the screen\n");
    fb.write_string("  echo    - Echo arguments to screen\n");
    fb.write_string("  memory  - Display memory statistics\n");
    fb.write_string("  ps      - Display process/task list\n");
    fb.write_string("  power   - Display/control power management\n");
    fb.write_string("  exit    - Exit the shell\n");
    Ok(())
}

/// Clear the screen
fn cmd_clear() -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    fb.clear();
    Ok(())
}

/// Echo arguments to screen
fn cmd_echo(args: Vec<&str>) -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            fb.write_string(" ");
        }
        fb.write_string(arg);
    }
    fb.write_string("\n");
    
    Ok(())
}

/// Display memory statistics
fn cmd_memory() -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    let stats = memory::stats::stats();
    
    fb.write_string("Memory Statistics:\n");
    
    // Total physical memory
    let total_phys = stats.total_physical();
    fb.write_string("  Total Physical: ");
    write_size(&mut fb, total_phys);
    fb.write_string("\n");
    
    // Used physical memory
    let used_phys = stats.used_physical();
    fb.write_string("  Used Physical:  ");
    write_size(&mut fb, used_phys);
    fb.write_string("\n");
    
    // Free physical memory
    let free_phys = total_phys - used_phys;
    fb.write_string("  Free Physical:  ");
    write_size(&mut fb, free_phys);
    fb.write_string("\n");
    
    // Total heap memory
    let total_heap = stats.total_heap();
    fb.write_string("  Total Heap:     ");
    write_size(&mut fb, total_heap);
    fb.write_string("\n");
    
    // Used heap memory
    let used_heap = stats.used_heap();
    fb.write_string("  Used Heap:      ");
    write_size(&mut fb, used_heap);
    fb.write_string("\n");
    
    // Allocations and deallocations
    fb.write_string("  Allocations:    ");
    write_number(&mut fb, stats.heap_allocations());
    fb.write_string("\n");
    
    fb.write_string("  Deallocations:  ");
    write_number(&mut fb, stats.heap_deallocations());
    fb.write_string("\n");
    
    Ok(())
}

/// Display process/task list
fn cmd_ps() -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    
    fb.write_string("Task List:\n");
    fb.write_string("  ID    NAME                STATE       PRIORITY\n");
    fb.write_string("  ----  ------------------  ----------  --------\n");
    
    // Access the scheduler
    let scheduler = task::scheduler::scheduler();
    
    // Get task count
    let task_count = scheduler.total_task_count();
    
    if task_count == 0 {
        fb.write_string("  No tasks running.\n");
    } else {
        // Iterate through tasks (simplified - we'll just show count for now)
        fb.write_string("  Total tasks: ");
        write_number(&mut fb, task_count);
        fb.write_string("\n");
        fb.write_string("  Ready tasks: ");
        write_number(&mut fb, scheduler.ready_task_count());
        fb.write_string("\n");
    }
    
    Ok(())
}

/// Exit the shell
fn cmd_exit(shell: &mut super::Shell) -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    fb.write_string("Exiting shell...\n");
    shell.stop();
    Ok(())
}

/// Helper function to write a memory size in human-readable format
fn write_size(fb: &mut framebuffer::FramebufferWriter, size: usize) {
    if size >= 1024 * 1024 * 1024 {
        // GiB
        let gib = size / (1024 * 1024 * 1024);
        write_number(fb, gib);
        fb.write_string(" GiB");
    } else if size >= 1024 * 1024 {
        // MiB
        let mib = size / (1024 * 1024);
        write_number(fb, mib);
        fb.write_string(" MiB");
    } else if size >= 1024 {
        // KiB
        let kib = size / 1024;
        write_number(fb, kib);
        fb.write_string(" KiB");
    } else {
        // Bytes
        write_number(fb, size);
        fb.write_string(" B");
    }
}

/// Helper function to write a number to framebuffer
fn write_number(fb: &mut framebuffer::FramebufferWriter, num: usize) {
    let mut buf = [0u8; 20];
    let mut n = num;
    let mut i = 0;
    
    if n == 0 {
        fb.write_string("0");
        return;
    }
    
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    
    // Reverse and write
    while i > 0 {
        i -= 1;
        let byte_slice = &[buf[i]];
        let s = core::str::from_utf8(byte_slice).unwrap_or("?");
        fb.write_string(s);
    }
}

/// Power management command
fn cmd_power(args: Vec<&str>) -> Result<(), &'static str> {
    let mut fb = framebuffer::framebuffer();
    
    if args.is_empty() {
        // Display power status
        let summary = power::get_power_summary();
        
        fb.write_string("Power Management Status:\n\n");
        
        // CPU Power State
        fb.write_string("CPU:\n");
        fb.write_string("  P-state: ");
        fb.write_string(match summary.cpu.p_state {
            power::PState::P0 => "P0 (Max Performance)\n",
            power::PState::P1 => "P1 (Medium Performance)\n",
            power::PState::P2 => "P2 (Low Performance)\n",
            power::PState::P3 => "P3 (Min Performance)\n",
        });
        
        fb.write_string("  C-state: ");
        fb.write_string(match summary.cpu.c_state {
            power::CState::C0 => "C0 (Running)\n",
            power::CState::C1 => "C1 (Halt)\n",
            power::CState::C2 => "C2 (Stop-Clock)\n",
            power::CState::C3 => "C3 (Deep Sleep)\n",
        });
        
        fb.write_string("  Policy: ");
        fb.write_string(match summary.cpu.policy {
            power::ScalingPolicy::Performance => "Performance\n",
            power::ScalingPolicy::Balanced => "Balanced\n",
            power::ScalingPolicy::PowerSave => "PowerSave\n",
        });
        
        fb.write_string("  Frequency: ");
        write_number(&mut fb, summary.cpu.frequency_mhz as usize);
        fb.write_string(" MHz\n\n");
        
        // System Sleep State
        fb.write_string("System:\n");
        fb.write_string("  Sleep State: ");
        fb.write_string(match summary.sleep_state {
            power::SleepState::S0 => "S0 (Working)\n",
            power::SleepState::S1 => "S1 (Standby)\n",
            power::SleepState::S3 => "S3 (Suspend to RAM)\n",
            power::SleepState::S4 => "S4 (Hibernate)\n",
            power::SleepState::S5 => "S5 (Soft Off)\n",
        });
        
        // Battery Status
        fb.write_string("\nBattery:\n");
        if summary.battery.is_present {
            fb.write_string("  Status: ");
            fb.write_string(match summary.battery.status {
                power::BatteryStatus::Charging => "Charging\n",
                power::BatteryStatus::Discharging => "Discharging\n",
                power::BatteryStatus::Full => "Full\n",
                power::BatteryStatus::NotPresent => "Not Present\n",
                power::BatteryStatus::Unknown => "Unknown\n",
            });
            
            fb.write_string("  Charge: ");
            write_number(&mut fb, summary.battery.charge_percent as usize);
            fb.write_string("%\n");
        } else {
            fb.write_string("  Not Present (AC Power)\n");
        }
        
        fb.write_string("\nUsage: power [status|policy|devices]\n");
        
    } else {
        match args[0] {
            "status" => {
                fb.write_string("Power Status: ");
                fb.write_string(match power::suspend::get_sleep_state() {
                    power::SleepState::S0 => "Active (S0)\n",
                    power::SleepState::S1 => "Standby (S1)\n",
                    power::SleepState::S3 => "Suspended (S3)\n",
                    power::SleepState::S4 => "Hibernated (S4)\n",
                    power::SleepState::S5 => "Off (S5)\n",
                });
            }
            "policy" => {
                if args.len() > 1 {
                    // Set policy
                    match args[1] {
                        "performance" => {
                            power::cpu::set_scaling_policy(power::ScalingPolicy::Performance)?;
                            fb.write_string("CPU scaling policy set to: Performance\n");
                        }
                        "balanced" => {
                            power::cpu::set_scaling_policy(power::ScalingPolicy::Balanced)?;
                            fb.write_string("CPU scaling policy set to: Balanced\n");
                        }
                        "powersave" => {
                            power::cpu::set_scaling_policy(power::ScalingPolicy::PowerSave)?;
                            fb.write_string("CPU scaling policy set to: PowerSave\n");
                        }
                        _ => {
                            fb.write_string("Invalid policy. Use: performance, balanced, or powersave\n");
                        }
                    }
                } else {
                    fb.write_string("Current policy: ");
                    fb.write_string(match power::cpu::get_scaling_policy() {
                        power::ScalingPolicy::Performance => "Performance\n",
                        power::ScalingPolicy::Balanced => "Balanced\n",
                        power::ScalingPolicy::PowerSave => "PowerSave\n",
                    });
                }
            }
            "devices" => {
                fb.write_string("Registered Power-Managed Devices:\n");
                let devices = power::device::list_devices();
                if devices.is_empty() {
                    fb.write_string("  No devices registered\n");
                } else {
                    for device_tuple in devices.iter() {
                        let (name, state) = device_tuple;
                        fb.write_string("  - ");
                        fb.write_string(name);
                        fb.write_string(": ");
                        fb.write_string(match state {
                            power::DevicePowerState::D0 => "D0 (On)\n",
                            power::DevicePowerState::D1 => "D1 (Low Power)\n",
                            power::DevicePowerState::D2 => "D2 (Lower Power)\n",
                            power::DevicePowerState::D3 => "D3 (Off)\n",
                        });
                    }
                }
            }
            _ => {
                fb.write_string("Unknown power subcommand. Use: status, policy, or devices\n");
            }
        }
    }
    
    Ok(())
}
