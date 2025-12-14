//! Example Tasks for Multi-tasking Demo
//!
//! This module provides example task functions that can be used to demonstrate
//! concurrent process execution and scheduling.

/// Task 1: Counter task
///
/// This task counts from 0 to a limit and prints each value.
#[no_mangle]
pub extern "C" fn task1() -> ! {
    #[cfg(not(test))]
    {
        let mut counter = 0u64;
        loop {
            fanga_arch_x86_64::serial_println!("[Task 1] Count: {}", counter);
            counter += 1;
            
            // Simple busy-wait delay
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
            
            if counter >= 10 {
                // Exit after counting to 10
                fanga_arch_x86_64::serial_println!("[Task 1] Exiting...");
                // In a real implementation, we would call exit() syscall here
                break;
            }
        }
    }
    
    // Halt if we exit
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}

/// Task 2: Computation task
///
/// This task performs simple computations and prints results.
#[no_mangle]
pub extern "C" fn task2() -> ! {
    #[cfg(not(test))]
    {
        let mut result = 0u64;
        for i in 0..10 {
            result += i * i;
            fanga_arch_x86_64::serial_println!("[Task 2] Sum of squares up to {}: {}", i, result);
            
            // Simple busy-wait delay
            for _ in 0..1000000 {
                core::hint::spin_loop();
            }
        }
        
        fanga_arch_x86_64::serial_println!("[Task 2] Final result: {}", result);
        fanga_arch_x86_64::serial_println!("[Task 2] Exiting...");
    }
    
    // Halt if we exit
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}

/// Task 3: Low priority background task
///
/// This task runs in the background with lower priority.
#[no_mangle]
pub extern "C" fn task3() -> ! {
    #[cfg(not(test))]
    {
        let mut heartbeat = 0u64;
        loop {
            fanga_arch_x86_64::serial_println!("[Task 3] Heartbeat: {}", heartbeat);
            heartbeat += 1;
            
            // Longer delay for background task
            for _ in 0..2000000 {
                core::hint::spin_loop();
            }
            
            if heartbeat >= 5 {
                fanga_arch_x86_64::serial_println!("[Task 3] Exiting...");
                break;
            }
        }
    }
    
    // Halt if we exit
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}

/// Idle task - runs when no other tasks are ready
///
/// This task has the lowest priority and simply yields the CPU.
#[no_mangle]
pub extern "C" fn idle_task() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}

/// Task 4: Timer demonstration task
///
/// This task demonstrates the timer and time management system by:
/// - Printing system uptime
/// - Using delays
/// - Tracking elapsed time
#[no_mangle]
pub extern "C" fn timer_demo_task() -> ! {
    #[cfg(not(test))]
    {
        fanga_arch_x86_64::serial_println!("[Timer Demo] Starting timer demonstration...");
        
        // Print initial uptime
        let start_ticks = fanga_arch_x86_64::interrupts::idt::timer_ticks();
        let start_ms = fanga_arch_x86_64::interrupts::idt::uptime_ms();
        fanga_arch_x86_64::serial_println!("[Timer Demo] Start: {} ticks, {} ms", start_ticks, start_ms);
        
        // Demonstrate delays
        for i in 0..5 {
            let before = fanga_arch_x86_64::interrupts::idt::uptime_ms();
            
            fanga_arch_x86_64::serial_println!("[Timer Demo] Iteration {}: Delaying 100ms...", i);
            crate::task::time::delay_ms(100);
            
            let after = fanga_arch_x86_64::interrupts::idt::uptime_ms();
            let elapsed = after - before;
            
            fanga_arch_x86_64::serial_println!(
                "[Timer Demo] Iteration {}: Delay complete! Elapsed: {}ms",
                i, elapsed
            );
        }
        
        // Print final uptime
        let end_ticks = fanga_arch_x86_64::interrupts::idt::timer_ticks();
        let end_ms = fanga_arch_x86_64::interrupts::idt::uptime_ms();
        let total_elapsed = end_ms - start_ms;
        
        fanga_arch_x86_64::serial_println!(
            "[Timer Demo] End: {} ticks, {} ms (elapsed: {} ms)",
            end_ticks, end_ms, total_elapsed
        );
        
        fanga_arch_x86_64::serial_println!("[Timer Demo] Demonstration complete!");
    }
    
    // Halt if we exit
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}
