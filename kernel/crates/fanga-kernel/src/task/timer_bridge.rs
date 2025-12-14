//! Timer Interrupt Bridge
//!
//! This module provides the bridge between the arch-specific timer interrupt
//! and the kernel's scheduler, enabling preemptive multitasking.

use crate::task::sched_timer;

/// Timer interrupt callback that will be called from the arch timer IRQ handler
/// 
/// This function is called on each timer tick and triggers the scheduler
/// to perform preemptive task switching.
pub fn timer_callback() {
    // Call the scheduler's timer-based scheduling logic
    sched_timer::schedule_on_timer();
}

/// Initialize the timer interrupt system
/// 
/// This registers the timer callback with the arch layer so that
/// the scheduler is invoked on each timer interrupt.
pub fn init() {
    unsafe {
        fanga_arch_x86_64::interrupts::idt::set_timer_callback(timer_callback);
    }
}
