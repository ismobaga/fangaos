// Minimal libc for FangaOS user programs
// This provides basic syscall wrappers and startup code

// Syscall numbers (must match kernel)
const SYS_READ: u64 = 0;
const SYS_WRITE: u64 = 1;
const SYS_OPEN: u64 = 2;
const SYS_CLOSE: u64 = 3;
const SYS_EXIT: u64 = 60;

// Syscall wrapper - uses the syscall instruction
#[inline(always)]
unsafe fn syscall1(num: u64, arg1: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        lateout("rax") ret,
        out("rcx") _,  // clobbered by syscall
        out("r11") _,  // clobbered by syscall
        options(nostack)
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        out("rcx") _,  // clobbered by syscall
        out("r11") _,  // clobbered by syscall
        options(nostack)
    );
    ret
}

// Write to file descriptor
pub fn write(fd: i32, buf: &[u8]) -> i64 {
    unsafe {
        syscall3(
            SYS_WRITE,
            fd as u64,
            buf.as_ptr() as u64,
            buf.len() as u64,
        )
    }
}

// Read from file descriptor
pub fn read(fd: i32, buf: &mut [u8]) -> i64 {
    unsafe {
        syscall3(
            SYS_READ,
            fd as u64,
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
        )
    }
}

// Open file
pub fn open(path: &str, flags: i32, mode: i32) -> i64 {
    unsafe {
        syscall3(
            SYS_OPEN,
            path.as_ptr() as u64,
            flags as u64,
            mode as u64,
        )
    }
}

// Close file descriptor
pub fn close(fd: i32) -> i64 {
    unsafe {
        syscall1(SYS_CLOSE, fd as u64)
    }
}

// Exit the program
pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    // Should never reach here, but just in case
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// Print a string to stdout
pub fn print(s: &str) {
    write(1, s.as_bytes());
}

// Print a string with newline
pub fn println(s: &str) {
    print(s);
    print("\n");
}
