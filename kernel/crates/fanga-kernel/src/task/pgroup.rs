//! Process Groups and Sessions
//!
//! This module implements POSIX-like process groups and sessions for job control:
//! - Process groups: Groups of related processes
//! - Sessions: Collection of process groups
//! - Job control: Foreground/background process management
//! - Controlling terminal: Terminal associated with a session

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use super::tcb::TaskId;

/// Process Group ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessGroupId(pub usize);

impl ProcessGroupId {
    /// Create a new process group ID
    pub const fn new(id: usize) -> Self {
        ProcessGroupId(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// Session ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub usize);

impl SessionId {
    /// Create a new session ID
    pub const fn new(id: usize) -> Self {
        SessionId(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// Controlling terminal identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalId(pub usize);

impl TerminalId {
    /// Create a new terminal ID
    pub const fn new(id: usize) -> Self {
        TerminalId(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// Process group
#[derive(Debug)]
pub struct ProcessGroup {
    /// Process group ID
    pub id: ProcessGroupId,
    
    /// Session this group belongs to
    pub session_id: SessionId,
    
    /// Process group leader (first process in the group)
    pub leader: TaskId,
    
    /// All processes in this group
    pub members: Vec<TaskId>,
    
    /// Whether this is the foreground process group
    pub is_foreground: bool,
}

impl ProcessGroup {
    /// Create a new process group
    pub fn new(id: ProcessGroupId, session_id: SessionId, leader: TaskId) -> Self {
        Self {
            id,
            session_id,
            leader,
            members: vec![leader],
            is_foreground: false,
        }
    }
    
    /// Add a process to the group
    pub fn add_member(&mut self, pid: TaskId) -> Result<(), &'static str> {
        if self.members.contains(&pid) {
            return Err("Process already in group");
        }
        self.members.push(pid);
        Ok(())
    }
    
    /// Remove a process from the group
    pub fn remove_member(&mut self, pid: TaskId) -> Result<(), &'static str> {
        if let Some(pos) = self.members.iter().position(|&p| p == pid) {
            self.members.remove(pos);
            Ok(())
        } else {
            Err("Process not in group")
        }
    }
    
    /// Check if the group is empty
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
    
    /// Get the number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
    
    /// Set as foreground group
    pub fn set_foreground(&mut self) {
        self.is_foreground = true;
    }
    
    /// Set as background group
    pub fn set_background(&mut self) {
        self.is_foreground = false;
    }
}

/// Session
#[derive(Debug)]
pub struct Session {
    /// Session ID
    pub id: SessionId,
    
    /// Session leader (process that created the session)
    pub leader: TaskId,
    
    /// Process groups in this session
    pub process_groups: Vec<ProcessGroupId>,
    
    /// Controlling terminal (if any)
    pub controlling_terminal: Option<TerminalId>,
    
    /// Foreground process group (if any)
    pub foreground_group: Option<ProcessGroupId>,
}

impl Session {
    /// Create a new session
    pub fn new(id: SessionId, leader: TaskId) -> Self {
        Self {
            id,
            leader,
            process_groups: Vec::new(),
            controlling_terminal: None,
            foreground_group: None,
        }
    }
    
    /// Add a process group to the session
    pub fn add_process_group(&mut self, pgid: ProcessGroupId) -> Result<(), &'static str> {
        if self.process_groups.contains(&pgid) {
            return Err("Process group already in session");
        }
        self.process_groups.push(pgid);
        Ok(())
    }
    
    /// Remove a process group from the session
    pub fn remove_process_group(&mut self, pgid: ProcessGroupId) -> Result<(), &'static str> {
        if let Some(pos) = self.process_groups.iter().position(|&p| p == pgid) {
            self.process_groups.remove(pos);
            
            // Clear foreground group if it's being removed
            if self.foreground_group == Some(pgid) {
                self.foreground_group = None;
            }
            
            Ok(())
        } else {
            Err("Process group not in session")
        }
    }
    
    /// Set the controlling terminal
    pub fn set_controlling_terminal(&mut self, terminal: TerminalId) {
        self.controlling_terminal = Some(terminal);
    }
    
    /// Clear the controlling terminal
    pub fn clear_controlling_terminal(&mut self) {
        self.controlling_terminal = None;
    }
    
    /// Set the foreground process group
    pub fn set_foreground_group(&mut self, pgid: ProcessGroupId) -> Result<(), &'static str> {
        if !self.process_groups.contains(&pgid) {
            return Err("Process group not in session");
        }
        self.foreground_group = Some(pgid);
        Ok(())
    }
    
    /// Clear the foreground process group
    pub fn clear_foreground_group(&mut self) {
        self.foreground_group = None;
    }
    
    /// Check if the session has a controlling terminal
    pub fn has_controlling_terminal(&self) -> bool {
        self.controlling_terminal.is_some()
    }
    
    /// Get the number of process groups
    pub fn process_group_count(&self) -> usize {
        self.process_groups.len()
    }
}

/// Process group and session manager
pub struct ProcessGroupManager {
    /// All process groups
    process_groups: BTreeMap<ProcessGroupId, ProcessGroup>,
    
    /// All sessions
    sessions: BTreeMap<SessionId, Session>,
    
    /// Process to process group mapping
    process_to_group: BTreeMap<TaskId, ProcessGroupId>,
    
    /// Process group to session mapping
    group_to_session: BTreeMap<ProcessGroupId, SessionId>,
    
    /// Next process group ID
    next_pgid: usize,
    
    /// Next session ID
    next_sid: usize,
}

impl ProcessGroupManager {
    /// Create a new process group manager
    pub fn new() -> Self {
        Self {
            process_groups: BTreeMap::new(),
            sessions: BTreeMap::new(),
            process_to_group: BTreeMap::new(),
            group_to_session: BTreeMap::new(),
            next_pgid: 1,
            next_sid: 1,
        }
    }
    
    /// Create a new session with the given process as leader
    /// The process becomes the session leader and process group leader
    pub fn create_session(&mut self, leader: TaskId) -> Result<SessionId, &'static str> {
        // Check if process is already a session/group leader
        if let Some(pgid) = self.process_to_group.get(&leader) {
            if let Some(group) = self.process_groups.get(pgid) {
                if group.leader == leader {
                    return Err("Process is already a process group leader");
                }
            }
        }
        
        let sid = SessionId::new(self.next_sid);
        self.next_sid += 1;
        
        let pgid = ProcessGroupId::new(self.next_pgid);
        self.next_pgid += 1;
        
        // Create session
        let mut session = Session::new(sid, leader);
        session.add_process_group(pgid).unwrap();
        
        // Create process group
        let process_group = ProcessGroup::new(pgid, sid, leader);
        
        // Update mappings
        self.sessions.insert(sid, session);
        self.process_groups.insert(pgid, process_group);
        self.process_to_group.insert(leader, pgid);
        self.group_to_session.insert(pgid, sid);
        
        Ok(sid)
    }
    
    /// Create a new process group in an existing session
    pub fn create_process_group(&mut self, leader: TaskId, session_id: SessionId) -> Result<ProcessGroupId, &'static str> {
        // Verify session exists
        if !self.sessions.contains_key(&session_id) {
            return Err("Session not found");
        }
        
        let pgid = ProcessGroupId::new(self.next_pgid);
        self.next_pgid += 1;
        
        let process_group = ProcessGroup::new(pgid, session_id, leader);
        
        // Update mappings
        self.process_groups.insert(pgid, process_group);
        self.process_to_group.insert(leader, pgid);
        self.group_to_session.insert(pgid, session_id);
        
        // Add to session
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.add_process_group(pgid).unwrap();
        }
        
        Ok(pgid)
    }
    
    /// Add a process to an existing process group
    pub fn add_to_process_group(&mut self, pid: TaskId, pgid: ProcessGroupId) -> Result<(), &'static str> {
        // Get the process group
        let group = self.process_groups.get_mut(&pgid).ok_or("Process group not found")?;
        
        // Add process to group
        group.add_member(pid)?;
        
        // Update mapping
        self.process_to_group.insert(pid, pgid);
        
        Ok(())
    }
    
    /// Remove a process from its process group
    pub fn remove_from_process_group(&mut self, pid: TaskId) -> Result<(), &'static str> {
        let pgid = self.process_to_group.remove(&pid).ok_or("Process not in any group")?;
        
        if let Some(group) = self.process_groups.get_mut(&pgid) {
            group.remove_member(pid)?;
            
            // If group is empty and not the session leader, remove the group
            if group.is_empty() {
                let sid = self.group_to_session.remove(&pgid).unwrap();
                self.process_groups.remove(&pgid);
                
                if let Some(session) = self.sessions.get_mut(&sid) {
                    let _ = session.remove_process_group(pgid);
                    
                    // If session has no groups, remove it
                    if session.process_group_count() == 0 {
                        self.sessions.remove(&sid);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the process group ID for a process
    pub fn get_process_group(&self, pid: TaskId) -> Option<ProcessGroupId> {
        self.process_to_group.get(&pid).copied()
    }
    
    /// Get the session ID for a process
    pub fn get_session(&self, pid: TaskId) -> Option<SessionId> {
        let pgid = self.get_process_group(pid)?;
        self.group_to_session.get(&pgid).copied()
    }
    
    /// Get a reference to a process group
    pub fn get_group(&self, pgid: ProcessGroupId) -> Option<&ProcessGroup> {
        self.process_groups.get(&pgid)
    }
    
    /// Get a mutable reference to a process group
    pub fn get_group_mut(&mut self, pgid: ProcessGroupId) -> Option<&mut ProcessGroup> {
        self.process_groups.get_mut(&pgid)
    }
    
    /// Get a reference to a session
    pub fn get_session_ref(&self, sid: SessionId) -> Option<&Session> {
        self.sessions.get(&sid)
    }
    
    /// Get a mutable reference to a session
    pub fn get_session_mut(&mut self, sid: SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(&sid)
    }
    
    /// Set a process group as foreground
    pub fn set_foreground(&mut self, pgid: ProcessGroupId) -> Result<(), &'static str> {
        let sid = self.group_to_session.get(&pgid).ok_or("Process group not found")?;
        
        // Clear foreground flag from all groups in the session
        for (group_pgid, group_sid) in &self.group_to_session {
            if group_sid == sid {
                if let Some(group) = self.process_groups.get_mut(group_pgid) {
                    group.set_background();
                }
            }
        }
        
        // Set the new foreground group
        if let Some(group) = self.process_groups.get_mut(&pgid) {
            group.set_foreground();
        }
        
        // Update session
        if let Some(session) = self.sessions.get_mut(sid) {
            session.set_foreground_group(pgid)?;
        }
        
        Ok(())
    }
    
    /// Get all processes in a process group
    pub fn get_group_members(&self, pgid: ProcessGroupId) -> Vec<TaskId> {
        self.process_groups
            .get(&pgid)
            .map(|g| g.members.clone())
            .unwrap_or_default()
    }
}

impl Default for ProcessGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let mut manager = ProcessGroupManager::new();
        let leader = TaskId::new(1);
        
        let sid = manager.create_session(leader).unwrap();
        assert_eq!(sid, SessionId::new(1));
        
        // Verify session was created
        let session = manager.get_session_ref(sid).unwrap();
        assert_eq!(session.leader, leader);
        assert_eq!(session.process_group_count(), 1);
        
        // Verify process group was created
        let pgid = manager.get_process_group(leader).unwrap();
        let group = manager.get_group(pgid).unwrap();
        assert_eq!(group.leader, leader);
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_create_process_group() {
        let mut manager = ProcessGroupManager::new();
        let session_leader = TaskId::new(1);
        let group_leader = TaskId::new(2);
        
        let sid = manager.create_session(session_leader).unwrap();
        let pgid = manager.create_process_group(group_leader, sid).unwrap();
        
        // Verify process group was added to session
        let session = manager.get_session_ref(sid).unwrap();
        assert_eq!(session.process_group_count(), 2);
        
        // Verify process group was created
        let group = manager.get_group(pgid).unwrap();
        assert_eq!(group.leader, group_leader);
        assert_eq!(group.session_id, sid);
    }

    #[test]
    fn test_add_to_process_group() {
        let mut manager = ProcessGroupManager::new();
        let leader = TaskId::new(1);
        let member = TaskId::new(2);
        
        let sid = manager.create_session(leader).unwrap();
        let pgid = manager.get_process_group(leader).unwrap();
        
        manager.add_to_process_group(member, pgid).unwrap();
        
        // Verify member was added
        let group = manager.get_group(pgid).unwrap();
        assert_eq!(group.member_count(), 2);
        assert!(group.members.contains(&member));
        
        // Verify mapping
        assert_eq!(manager.get_process_group(member), Some(pgid));
        assert_eq!(manager.get_session(member), Some(sid));
    }

    #[test]
    fn test_remove_from_process_group() {
        let mut manager = ProcessGroupManager::new();
        let leader = TaskId::new(1);
        let member = TaskId::new(2);
        
        manager.create_session(leader).unwrap();
        let pgid = manager.get_process_group(leader).unwrap();
        manager.add_to_process_group(member, pgid).unwrap();
        
        // Remove member
        manager.remove_from_process_group(member).unwrap();
        
        // Verify member was removed
        let group = manager.get_group(pgid).unwrap();
        assert_eq!(group.member_count(), 1);
        assert!(!group.members.contains(&member));
    }

    #[test]
    fn test_foreground_group() {
        let mut manager = ProcessGroupManager::new();
        let leader = TaskId::new(1);
        
        manager.create_session(leader).unwrap();
        let pgid = manager.get_process_group(leader).unwrap();
        
        // Set as foreground
        manager.set_foreground(pgid).unwrap();
        
        let group = manager.get_group(pgid).unwrap();
        assert!(group.is_foreground);
        
        let sid = manager.get_session(leader).unwrap();
        let session = manager.get_session_ref(sid).unwrap();
        assert_eq!(session.foreground_group, Some(pgid));
    }

    #[test]
    fn test_controlling_terminal() {
        let mut manager = ProcessGroupManager::new();
        let leader = TaskId::new(1);
        
        let sid = manager.create_session(leader).unwrap();
        let terminal = TerminalId::new(1);
        
        // Set controlling terminal
        let session = manager.get_session_mut(sid).unwrap();
        session.set_controlling_terminal(terminal);
        assert!(session.has_controlling_terminal());
        assert_eq!(session.controlling_terminal, Some(terminal));
        
        // Clear controlling terminal
        session.clear_controlling_terminal();
        assert!(!session.has_controlling_terminal());
    }
}
