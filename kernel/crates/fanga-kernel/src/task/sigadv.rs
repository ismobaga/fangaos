//! Advanced Signal Handling
//!
//! This module extends basic signal support with full POSIX-like features:
//! - Signal actions and custom handlers
//! - Signal masks (sigprocmask)
//! - Pending signal tracking
//! - Real-time signals with queueing
//! - Signal delivery to process groups

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::tcb::TaskId;
use super::ipc::Signal;
use super::pgroup::ProcessGroupId;

/// Signal action disposition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    /// Default action (usually terminate)
    Default,
    /// Ignore the signal
    Ignore,
    /// Custom handler at this address
    Handler(u64),
    /// Core dump and terminate
    Core,
}

impl Default for SignalAction {
    fn default() -> Self {
        SignalAction::Default
    }
}

/// Signal flags (SA_* flags from POSIX)
#[derive(Debug, Clone, Copy)]
pub struct SignalFlags {
    /// Restart syscalls interrupted by signal
    pub sa_restart: bool,
    
    /// Don't add signal to mask while handler executes
    pub sa_nodefer: bool,
    
    /// Reset handler to default after invocation
    pub sa_resethand: bool,
    
    /// Signal handler uses extended info
    pub sa_siginfo: bool,
}

impl Default for SignalFlags {
    fn default() -> Self {
        Self {
            sa_restart: false,
            sa_nodefer: false,
            sa_resethand: false,
            sa_siginfo: false,
        }
    }
}

/// Signal information for real-time signals
#[derive(Debug, Clone, Copy)]
pub struct SignalInfo {
    /// Signal number
    pub signal: Signal,
    
    /// Signal code (reason)
    pub code: i32,
    
    /// Sending process ID
    pub sender_pid: TaskId,
    
    /// User data
    pub value: i32,
}

impl SignalInfo {
    /// Create new signal info
    pub fn new(signal: Signal, sender_pid: TaskId) -> Self {
        Self {
            signal,
            code: 0,
            sender_pid,
            value: 0,
        }
    }
    
    /// Create signal info with value
    pub fn with_value(signal: Signal, sender_pid: TaskId, value: i32) -> Self {
        Self {
            signal,
            code: 0,
            sender_pid,
            value,
        }
    }
}

/// Signal action structure (like struct sigaction)
#[derive(Debug, Clone)]
pub struct SigAction {
    /// Action to take
    pub action: SignalAction,
    
    /// Additional signals to block during handler
    pub mask: u64,
    
    /// Signal flags
    pub flags: SignalFlags,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            action: SignalAction::Default,
            mask: 0,
            flags: SignalFlags::default(),
        }
    }
}

impl SigAction {
    /// Create a new signal action with default disposition
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a signal action with a handler
    pub fn with_handler(handler: u64) -> Self {
        Self {
            action: SignalAction::Handler(handler),
            mask: 0,
            flags: SignalFlags::default(),
        }
    }
    
    /// Create a signal action to ignore the signal
    pub fn ignore() -> Self {
        Self {
            action: SignalAction::Ignore,
            mask: 0,
            flags: SignalFlags::default(),
        }
    }
}

/// Advanced signal handler for a process
#[derive(Debug)]
pub struct AdvancedSignalHandler {
    /// Pending standard signals (bit mask)
    pending: u64,
    
    /// Blocked signals (bit mask)
    blocked: u64,
    
    /// Signal actions for each signal
    actions: [SigAction; 32],
    
    /// Queue for real-time signals (signals 32-63)
    rt_queue: VecDeque<SignalInfo>,
    
    /// Saved signal mask (for sigsuspend)
    saved_mask: Option<u64>,
}

impl AdvancedSignalHandler {
    /// Create a new signal handler
    pub fn new() -> Self {
        Self {
            pending: 0,
            blocked: 0,
            actions: Default::default(),
            rt_queue: VecDeque::new(),
            saved_mask: None,
        }
    }
    
    /// Helper to get the bit mask for a signal
    fn signal_bit(signal: Signal) -> u64 {
        1u64 << (signal.num() as u64)
    }
    
    /// Send a standard signal
    pub fn send(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.pending |= bit;
    }
    
    /// Send a real-time signal with info
    pub fn send_rt(&mut self, info: SignalInfo) {
        // For RT signals, queue them
        if info.signal.num() >= 32 {
            self.rt_queue.push_back(info);
        } else {
            // Standard signal
            self.send(info.signal);
        }
    }
    
    /// Check if a signal is pending
    pub fn is_pending(&self, signal: Signal) -> bool {
        let bit = Self::signal_bit(signal);
        (self.pending & bit) != 0
    }
    
    /// Check if a signal is blocked
    pub fn is_blocked(&self, signal: Signal) -> bool {
        let bit = Self::signal_bit(signal);
        (self.blocked & bit) != 0
    }
    
    /// Block a signal (add to signal mask)
    pub fn block(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.blocked |= bit;
    }
    
    /// Unblock a signal (remove from signal mask)
    pub fn unblock(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.blocked &= !bit;
    }
    
    /// Set the signal mask (sigprocmask)
    pub fn set_mask(&mut self, mask: u64) {
        self.blocked = mask;
    }
    
    /// Get the current signal mask
    pub fn get_mask(&self) -> u64 {
        self.blocked
    }
    
    /// Get pending signals mask
    pub fn get_pending(&self) -> u64 {
        self.pending
    }
    
    /// Clear a pending signal
    pub fn clear(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.pending &= !bit;
    }
    
    /// Get the next unblocked pending signal
    pub fn next_unblocked(&mut self) -> Option<SignalInfo> {
        // First check real-time signals in the queue
        if let Some(info) = self.rt_queue.pop_front() {
            return Some(info);
        }
        
        // Then check standard signals
        let unblocked_pending = self.pending & !self.blocked;
        if unblocked_pending == 0 {
            return None;
        }
        
        // Find the first set bit
        let bit_pos = unblocked_pending.trailing_zeros() as u8;
        
        if let Some(signal) = Signal::from_num(bit_pos) {
            // Clear the signal
            self.clear(signal);
            
            Some(SignalInfo::new(signal, TaskId::new(0)))
        } else {
            None
        }
    }
    
    /// Check if there are any pending unblocked signals
    pub fn has_pending(&self) -> bool {
        ((self.pending & !self.blocked) != 0) || !self.rt_queue.is_empty()
    }
    
    /// Set signal action (sigaction)
    pub fn set_action(&mut self, signal: Signal, action: SigAction) -> Result<SigAction, &'static str> {
        let sig_num = signal.num() as usize;
        if sig_num >= 32 {
            return Err("Invalid signal number");
        }
        
        // Cannot change SIGKILL or SIGSTOP
        if signal == Signal::SIGKILL || signal == Signal::SIGSTOP {
            return Err("Cannot change SIGKILL or SIGSTOP");
        }
        
        let old_action = self.actions[sig_num].clone();
        self.actions[sig_num] = action;
        
        Ok(old_action)
    }
    
    /// Get signal action
    pub fn get_action(&self, signal: Signal) -> Result<&SigAction, &'static str> {
        let sig_num = signal.num() as usize;
        if sig_num >= 32 {
            return Err("Invalid signal number");
        }
        
        Ok(&self.actions[sig_num])
    }
    
    /// Save current mask and set a new one (sigsuspend)
    pub fn suspend_with_mask(&mut self, mask: u64) {
        self.saved_mask = Some(self.blocked);
        self.blocked = mask;
    }
    
    /// Restore saved mask
    pub fn restore_mask(&mut self) {
        if let Some(mask) = self.saved_mask.take() {
            self.blocked = mask;
        }
    }
    
    /// Get default action for a signal
    pub fn default_action(signal: Signal) -> SignalAction {
        match signal {
            Signal::SIGKILL | Signal::SIGSTOP => SignalAction::Default,
            Signal::SIGCHLD | Signal::SIGCONT => SignalAction::Ignore,
            Signal::SIGQUIT | Signal::SIGILL | Signal::SIGABRT |
            Signal::SIGFPE | Signal::SIGSEGV | Signal::SIGBUS => SignalAction::Core,
            _ => SignalAction::Default,
        }
    }
    
    /// Get the number of pending RT signals
    pub fn rt_pending_count(&self) -> usize {
        self.rt_queue.len()
    }
}

impl Default for AdvancedSignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal delivery target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalTarget {
    /// Single process
    Process(TaskId),
    /// All processes in a process group
    ProcessGroup(ProcessGroupId),
    /// All processes in the system (broadcast)
    All,
}

/// Signal manager for system-wide signal operations
pub struct SignalManager {
    /// Process signal handlers
    handlers: Vec<Option<AdvancedSignalHandler>>,
}

impl SignalManager {
    /// Create a new signal manager
    pub fn new(max_processes: usize) -> Self {
        let mut handlers = Vec::new();
        handlers.resize_with(max_processes, || None);
        
        Self { handlers }
    }
    
    /// Get or create signal handler for a process
    pub fn get_or_create_handler(&mut self, pid: TaskId) -> &mut AdvancedSignalHandler {
        let idx = pid.as_usize();
        if idx >= self.handlers.len() {
            self.handlers.resize_with(idx + 1, || None);
        }
        
        self.handlers[idx].get_or_insert_with(AdvancedSignalHandler::new)
    }
    
    /// Get signal handler for a process
    pub fn get_handler(&self, pid: TaskId) -> Option<&AdvancedSignalHandler> {
        self.handlers.get(pid.as_usize())?.as_ref()
    }
    
    /// Get mutable signal handler for a process
    pub fn get_handler_mut(&mut self, pid: TaskId) -> Option<&mut AdvancedSignalHandler> {
        self.handlers.get_mut(pid.as_usize())?.as_mut()
    }
    
    /// Send a signal to a target
    pub fn send_signal(&mut self, target: SignalTarget, signal: Signal, sender: TaskId) {
        match target {
            SignalTarget::Process(pid) => {
                if let Some(handler) = self.get_handler_mut(pid) {
                    handler.send(signal);
                }
            }
            SignalTarget::ProcessGroup(_pgid) => {
                // Would need to iterate all processes in the group
                // This requires integration with ProcessGroupManager
                // For now, we just implement the structure
            }
            SignalTarget::All => {
                // Send to all processes (except sender)
                for (idx, handler_opt) in self.handlers.iter_mut().enumerate() {
                    if idx != sender.as_usize() {
                        if let Some(handler) = handler_opt {
                            handler.send(signal);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_action() {
        let action = SigAction::with_handler(0x1000);
        assert_eq!(action.action, SignalAction::Handler(0x1000));
        
        let ignore = SigAction::ignore();
        assert_eq!(ignore.action, SignalAction::Ignore);
    }

    #[test]
    fn test_advanced_signal_handler() {
        let mut handler = AdvancedSignalHandler::new();
        
        // Send a signal
        handler.send(Signal::SIGINT);
        assert!(handler.is_pending(Signal::SIGINT));
        assert!(handler.has_pending());
        
        // Get next signal
        let info = handler.next_unblocked().unwrap();
        assert_eq!(info.signal, Signal::SIGINT);
        assert!(!handler.is_pending(Signal::SIGINT));
    }

    #[test]
    fn test_signal_blocking() {
        let mut handler = AdvancedSignalHandler::new();
        
        // Block a signal
        handler.block(Signal::SIGINT);
        assert!(handler.is_blocked(Signal::SIGINT));
        
        // Send the blocked signal
        handler.send(Signal::SIGINT);
        assert!(handler.is_pending(Signal::SIGINT));
        
        // Should not be delivered
        assert!(!handler.has_pending());
        
        // Unblock
        handler.unblock(Signal::SIGINT);
        assert!(handler.has_pending());
    }

    #[test]
    fn test_signal_mask() {
        let mut handler = AdvancedSignalHandler::new();
        
        let mask = (1u64 << Signal::SIGINT.num()) | (1u64 << Signal::SIGTERM.num());
        handler.set_mask(mask);
        
        assert_eq!(handler.get_mask(), mask);
        assert!(handler.is_blocked(Signal::SIGINT));
        assert!(handler.is_blocked(Signal::SIGTERM));
        assert!(!handler.is_blocked(Signal::SIGUSR1));
    }

    #[test]
    fn test_sigaction() {
        let mut handler = AdvancedSignalHandler::new();
        
        let action = SigAction::with_handler(0x1000);
        let old = handler.set_action(Signal::SIGUSR1, action).unwrap();
        assert_eq!(old.action, SignalAction::Default);
        
        let current = handler.get_action(Signal::SIGUSR1).unwrap();
        assert_eq!(current.action, SignalAction::Handler(0x1000));
    }

    #[test]
    fn test_cannot_change_sigkill() {
        let mut handler = AdvancedSignalHandler::new();
        
        let action = SigAction::ignore();
        let result = handler.set_action(Signal::SIGKILL, action);
        assert!(result.is_err());
    }

    #[test]
    fn test_rt_signals() {
        let mut handler = AdvancedSignalHandler::new();
        
        let info = SignalInfo::with_value(Signal::SIGUSR1, TaskId::new(1), 42);
        handler.send_rt(info);
        
        assert_eq!(handler.rt_pending_count(), 0); // SIGUSR1 is not RT
        assert!(handler.has_pending());
    }

    #[test]
    fn test_sigsuspend() {
        let mut handler = AdvancedSignalHandler::new();
        
        let original_mask = (1u64 << Signal::SIGINT.num());
        handler.set_mask(original_mask);
        
        let suspend_mask = (1u64 << Signal::SIGTERM.num());
        handler.suspend_with_mask(suspend_mask);
        
        assert_eq!(handler.get_mask(), suspend_mask);
        
        handler.restore_mask();
        assert_eq!(handler.get_mask(), original_mask);
    }

    #[test]
    fn test_signal_info() {
        let info = SignalInfo::new(Signal::SIGUSR1, TaskId::new(42));
        assert_eq!(info.signal, Signal::SIGUSR1);
        assert_eq!(info.sender_pid, TaskId::new(42));
        assert_eq!(info.value, 0);
        
        let info2 = SignalInfo::with_value(Signal::SIGUSR2, TaskId::new(1), 123);
        assert_eq!(info2.value, 123);
    }
}
