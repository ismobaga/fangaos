/// Dynamic interrupt handler registration system
///
/// This module provides a system for dynamically registering interrupt handlers
/// at runtime, allowing device drivers and other kernel components to hook into
/// specific interrupt vectors.

use crate::interrupts::idt::{InterruptStackFrame, PIC1_OFFSET, PIC2_OFFSET};
use crate::interrupts::pic;

/// Type alias for interrupt handler functions
pub type InterruptHandler = fn(InterruptStackFrame);

/// Maximum number of handlers per interrupt vector
const MAX_HANDLERS_PER_VECTOR: usize = 4;

/// Handler entry in the registry
#[derive(Clone, Copy)]
struct HandlerEntry {
    handler: Option<InterruptHandler>,
}

impl HandlerEntry {
    const fn empty() -> Self {
        Self { handler: None }
    }
}

/// Registry for dynamically registered interrupt handlers
/// Supports up to 256 interrupt vectors, each with up to 4 handlers
struct HandlerRegistry {
    handlers: [[HandlerEntry; MAX_HANDLERS_PER_VECTOR]; 256],
}

impl HandlerRegistry {
    const fn new() -> Self {
        Self {
            handlers: [[HandlerEntry::empty(); MAX_HANDLERS_PER_VECTOR]; 256],
        }
    }

    /// Register a handler for a specific interrupt vector
    fn register(&mut self, vector: u8, handler: InterruptHandler) -> Result<(), &'static str> {
        let vector_handlers = &mut self.handlers[vector as usize];
        
        // Find an empty slot
        for entry in vector_handlers.iter_mut() {
            if entry.handler.is_none() {
                entry.handler = Some(handler);
                return Ok(());
            }
        }
        
        Err("Maximum number of handlers reached for this vector")
    }

    /// Unregister a handler for a specific interrupt vector
    fn unregister(&mut self, vector: u8, handler: InterruptHandler) -> Result<(), &'static str> {
        let vector_handlers = &mut self.handlers[vector as usize];
        
        for entry in vector_handlers.iter_mut() {
            if let Some(h) = entry.handler {
                if h as usize == handler as usize {
                    entry.handler = None;
                    return Ok(());
                }
            }
        }
        
        Err("Handler not found in registry")
    }

    /// Call all registered handlers for a vector
    #[allow(dead_code)]
    fn dispatch(&self, vector: u8, frame: InterruptStackFrame) {
        let vector_handlers = &self.handlers[vector as usize];
        
        for entry in vector_handlers.iter() {
            if let Some(handler) = entry.handler {
                handler(frame);
            }
        }
    }

    /// Check if a vector has any registered handlers
    #[allow(dead_code)]
    fn has_handlers(&self, vector: u8) -> bool {
        let vector_handlers = &self.handlers[vector as usize];
        vector_handlers.iter().any(|e| e.handler.is_some())
    }
}

static mut HANDLER_REGISTRY: HandlerRegistry = HandlerRegistry::new();

/// Register a custom interrupt handler for a specific vector
///
/// # Safety
/// The caller must ensure that:
/// - The handler function is safe to call from interrupt context
/// - The handler does not perform any blocking operations
/// - The handler completes quickly to avoid blocking other interrupts
pub unsafe fn register_handler(vector: u8, handler: InterruptHandler) -> Result<(), &'static str> {
    HANDLER_REGISTRY.register(vector, handler)
}

/// Unregister a previously registered interrupt handler
///
/// # Safety
/// The caller must ensure the handler is not currently executing
pub unsafe fn unregister_handler(vector: u8, handler: InterruptHandler) -> Result<(), &'static str> {
    HANDLER_REGISTRY.unregister(vector, handler)
}

/// Internal dispatcher called from the IDT entries
#[allow(dead_code)]
pub(crate) unsafe fn dispatch_handlers(vector: u8, frame: InterruptStackFrame) {
    HANDLER_REGISTRY.dispatch(vector, frame);
}

/// Register an IRQ handler (convenience function for PIC IRQs)
///
/// # Safety
/// Same safety requirements as register_handler
pub unsafe fn register_irq_handler(irq: u8, handler: InterruptHandler) -> Result<(), &'static str> {
    if irq < 8 {
        register_handler(PIC1_OFFSET + irq, handler)
    } else if irq < 16 {
        register_handler(PIC2_OFFSET + irq - 8, handler)
    } else {
        Err("Invalid IRQ number (must be 0-15)")
    }
}

/// Unregister an IRQ handler
///
/// # Safety
/// Same safety requirements as unregister_handler
pub unsafe fn unregister_irq_handler(irq: u8, handler: InterruptHandler) -> Result<(), &'static str> {
    if irq < 8 {
        unregister_handler(PIC1_OFFSET + irq, handler)
    } else if irq < 16 {
        unregister_handler(PIC2_OFFSET + irq - 8, handler)
    } else {
        Err("Invalid IRQ number (must be 0-15)")
    }
}

/// Enable an IRQ by unmasking it in the PIC
pub unsafe fn enable_irq(irq: u8) {
    pic::unmask_irq(irq);
}

/// Disable an IRQ by masking it in the PIC
pub unsafe fn disable_irq(irq: u8) {
    pic::mask_irq(irq);
}

/// Get the number of registered handlers for a vector
pub fn handler_count(vector: u8) -> usize {
    unsafe {
        HANDLER_REGISTRY.handlers[vector as usize]
            .iter()
            .filter(|e| e.handler.is_some())
            .count()
    }
}
