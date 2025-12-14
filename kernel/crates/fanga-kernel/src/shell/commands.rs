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

/// Execute a command
pub fn execute(command: &str, args: Vec<&str>, shell: &mut super::Shell) -> Result<(), &'static str> {
    match command {
        "" => Ok(()), // Empty command, do nothing
        "help" => cmd_help(),
        "clear" => cmd_clear(),
        "echo" => cmd_echo(args),
        "memory" => cmd_memory(),
        "ps" => cmd_ps(),
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
