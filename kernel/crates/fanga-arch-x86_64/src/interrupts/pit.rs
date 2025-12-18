/// Programmable Interval Timer (PIT) - Intel 8253/8254
///
/// The PIT is a hardware timer that generates periodic interrupts (IRQ0).
/// It operates at a base frequency of 1.193182 MHz (1193182 Hz).
///
/// This module provides configuration and management of the PIT timer
/// for system timing and preemptive multitasking.

use crate::port::{inb, outb};

/// PIT Channel 0 (used for system timer)
const PIT_CHANNEL_0: u16 = 0x40;

/// PIT Command/Mode register
const PIT_COMMAND: u16 = 0x43;

/// PIT base frequency in Hz (1.193182 MHz)
const PIT_BASE_FREQ: u32 = 1193182;

/// Default timer frequency in Hz (100 Hz = 10ms ticks)
pub const PIT_DEFAULT_FREQ: u32 = 100;

/// Command byte structure:
/// Bits 7-6: Channel select (00 = channel 0)
/// Bits 5-4: Access mode (11 = lobyte/hibyte)
/// Bits 3-1: Operating mode (011 = Mode 3, square wave)
/// Bit 0:    Binary/BCD mode (0 = binary)
const PIT_CMD_CHANNEL_0: u8 = 0b00 << 6;
const PIT_CMD_ACCESS_LOHI: u8 = 0b11 << 4;
const PIT_CMD_MODE_SQUARE: u8 = 0b011 << 1;
const PIT_CMD_BINARY: u8 = 0;

/// Calculate divisor for a given frequency
///
/// # Arguments
/// * `frequency` - Desired frequency in Hz
///
/// # Returns
/// The divisor value to program into the PIT
fn calculate_divisor(frequency: u32) -> u16 {
    if frequency == 0 {
        return 1;
    }
    
    let divisor = PIT_BASE_FREQ / frequency;
    
    // Clamp to valid range (1 to 65535)
    if divisor > 0xFFFF {
        0xFFFF
    } else if divisor < 1 {
        1
    } else {
        divisor as u16
    }
}

/// Initialize the PIT with the specified frequency
///
/// # Arguments
/// * `frequency` - Desired timer frequency in Hz (e.g., 100 Hz = 10ms ticks)
///
/// # Safety
/// This function performs direct I/O port access and should only be called
/// during system initialization with interrupts disabled.
pub unsafe fn init(frequency: u32) {
    let divisor = calculate_divisor(frequency);
    
    // Send command byte
    let command = PIT_CMD_CHANNEL_0 | PIT_CMD_ACCESS_LOHI | PIT_CMD_MODE_SQUARE | PIT_CMD_BINARY;
    outb(PIT_COMMAND, command);
    
    // Send divisor low byte
    outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
    
    // Send divisor high byte
    outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
}

/// Read the current counter value from the PIT
///
/// # Safety
/// This function performs direct I/O port access
pub unsafe fn read_counter() -> u16 {
    // Latch counter value
    outb(PIT_COMMAND, 0);
    
    // Read low byte then high byte
    let low = inb(PIT_CHANNEL_0) as u16;
    let high = inb(PIT_CHANNEL_0) as u16;
    
    (high << 8) | low
}

/// Get the frequency the PIT is currently configured for
///
/// # Arguments
/// * `divisor` - The divisor value programmed into the PIT
///
/// # Returns
/// The actual frequency in Hz
pub fn get_frequency(divisor: u16) -> u32 {
    if divisor == 0 {
        return PIT_BASE_FREQ;
    }
    PIT_BASE_FREQ / (divisor as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_divisor() {
        // Test standard frequencies
        assert_eq!(calculate_divisor(100), 11931);  // 100 Hz
        assert_eq!(calculate_divisor(1000), 1193); // 1 kHz
        assert_eq!(calculate_divisor(18), 65535);  // ~18.2 Hz (clamped to max)
        
        // Test edge cases
        assert_eq!(calculate_divisor(0), 1);       // Zero frequency
        assert_eq!(calculate_divisor(PIT_BASE_FREQ), 1); // Maximum frequency
        assert_eq!(calculate_divisor(1), 65535);   // Very low frequency clamped
    }
    
    #[test]
    fn test_get_frequency() {
        assert_eq!(get_frequency(11931), 100);
        assert_eq!(get_frequency(1193), 1000);
        assert_eq!(get_frequency(1), PIT_BASE_FREQ);
        assert_eq!(get_frequency(0), PIT_BASE_FREQ);
    }
    
    #[test]
    fn test_round_trip() {
        // Test that frequency -> divisor -> frequency gives approximately the same result
        // Note: Very low frequencies (< 19 Hz) are clamped to divisor 65535, so skip those
        for freq in [50, 100, 500, 1000, 10000] {
            let divisor = calculate_divisor(freq);
            let actual_freq = get_frequency(divisor);
            
            // Allow small error due to integer division
            let error = if freq > actual_freq {
                freq - actual_freq
            } else {
                actual_freq - freq
            };
            
            // Error should be less than 5% or 1 Hz (whichever is larger)
            let max_error = core::cmp::max(1, freq / 20);
            assert!(error <= max_error, 
                "Frequency {} Hz -> divisor {} -> {} Hz (error: {} Hz)",
                freq, divisor, actual_freq, error);
        }
    }
}
