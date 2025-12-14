# Inter-Process Communication (IPC)

## Overview

FangaOS provides comprehensive Inter-Process Communication (IPC) mechanisms that enable processes to communicate and synchronize their activities. The IPC subsystem includes message passing, pipes, shared memory, signals, and synchronization primitives.

## Architecture

The IPC implementation is located in `kernel/crates/fanga-kernel/src/task/ipc.rs` and provides the following components:

### Components

1. **Message Queues** - Asynchronous message passing between processes
2. **Pipes** - Unidirectional data streams (anonymous and named)
3. **Shared Memory** - Direct memory sharing between processes
4. **Signals** - Asynchronous notification mechanism
5. **Synchronization Primitives** - Mutexes and semaphores

## Message Passing

### Message Queue

Message queues provide a way for processes to exchange messages asynchronously.

```rust
pub struct MessageQueue {
    messages: VecDeque<Message>,
    max_size: usize,
    waiting_tasks: Vec<TaskId>,
}
```

**Features:**
- Maximum message size: 256 bytes
- FIFO ordering
- Blocking support for receivers
- Configurable queue size

**Usage:**

```rust
// Create a message queue
let mut queue = MessageQueue::new(10);

// Send a message
let msg = Message::new(sender_id, data)?;
queue.send(msg)?;

// Receive a message
if let Some(msg) = queue.receive() {
    process_message(msg);
}
```

**System Calls:**
- `SYS_MSGGET` (68) - Get/create message queue
- `SYS_MSGSND` (69) - Send message to queue
- `SYS_MSGRCV` (70) - Receive message from queue

## Pipes

Pipes provide unidirectional data flow between processes using a buffered channel.

### Pipe Structure

```rust
pub struct Pipe {
    buffer: VecDeque<u8>,
    max_size: usize,  // Default: 4096 bytes
    readers: usize,
    writers: usize,
    waiting_readers: Vec<TaskId>,
    waiting_writers: Vec<TaskId>,
}
```

**Features:**
- Fixed-size circular buffer (4KB default)
- Reference counting for readers/writers
- Non-blocking read/write operations
- EOF detection when no writers remain
- Broken pipe detection when no readers remain

**Behavior:**
- Read returns 0 when no data available and no writers (EOF)
- Write fails with error when no readers (EPIPE/broken pipe)
- Partial reads/writes when buffer is partially full/empty

**System Call:**
- `SYS_PIPE` (22) - Create a pipe
  - Returns two file descriptors: `pipefd[0]` (read end), `pipefd[1]` (write end)

**Example:**

```rust
let mut pipe = Pipe::new();

// Add reader and writer
pipe.add_reader();
pipe.add_writer();

// Write data
let data = b"Hello, pipe!";
let written = pipe.write(data)?;

// Read data
let mut buf = [0u8; 32];
let read = pipe.read(&mut buf)?;
```

## Shared Memory

Shared memory allows multiple processes to access the same physical memory region.

### Shared Memory Structure

```rust
pub struct SharedMemory {
    phys_addr: PhysAddr,
    size: usize,
    ref_count: usize,
    attached_tasks: Vec<TaskId>,
}
```

**Features:**
- Reference counting for automatic cleanup
- Track which processes have attached
- Physical address-based mapping
- Arbitrary size segments

**Lifecycle:**
1. **Create** - Allocate physical memory for sharing
2. **Attach** - Map segment into process address space
3. **Use** - Direct memory access (fastest IPC)
4. **Detach** - Unmap from process address space
5. **Destroy** - Free memory when ref_count reaches 0

**System Calls:**
- `SYS_SHMGET` (29) - Create/get shared memory segment
- `SYS_SHMAT` (30) - Attach shared memory segment
- `SYS_SHMDT` (67) - Detach shared memory segment
- `SYS_SHMCTL` (31) - Control shared memory segment

**Example:**

```rust
// Create shared memory
let mut shm = SharedMemory::new(phys_addr, 4096);

// Process 1 attaches
shm.attach(task1)?;

// Process 2 attaches
shm.attach(task2)?;

// Both processes can now access the shared memory

// Process 1 detaches
shm.detach(task1)?;

// Process 2 detaches (last reference)
shm.detach(task2)?;
```

## Signals

Signals provide an asynchronous notification mechanism for processes.

### Signal Types

FangaOS implements POSIX-like signals:

| Signal | Number | Description |
|--------|--------|-------------|
| SIGHUP | 1 | Hangup |
| SIGINT | 2 | Interrupt (Ctrl+C) |
| SIGQUIT | 3 | Quit |
| SIGILL | 4 | Illegal instruction |
| SIGTRAP | 5 | Trace/breakpoint trap |
| SIGABRT | 6 | Abort |
| SIGBUS | 7 | Bus error |
| SIGFPE | 8 | Floating point exception |
| SIGKILL | 9 | Kill (cannot be caught) |
| SIGUSR1 | 10 | User-defined signal 1 |
| SIGSEGV | 11 | Segmentation fault |
| SIGUSR2 | 12 | User-defined signal 2 |
| SIGPIPE | 13 | Broken pipe |
| SIGALRM | 14 | Alarm clock |
| SIGTERM | 15 | Termination |
| SIGCHLD | 17 | Child stopped or terminated |
| SIGCONT | 18 | Continue |
| SIGSTOP | 19 | Stop (cannot be caught) |
| SIGTSTP | 20 | Terminal stop |

### Signal Handler

```rust
pub struct SignalHandler {
    pending: u32,   // Bitmask of pending signals
    blocked: u32,   // Bitmask of blocked signals
}
```

**Features:**
- Multiple pending signals (bit mask)
- Signal blocking/unblocking
- Priority-based delivery (lower signal numbers first)
- Automatic signal clearing on delivery

**Operations:**
- `send()` - Send a signal to the handler
- `block()` - Block a signal from being delivered
- `unblock()` - Allow a blocked signal to be delivered
- `clear()` - Clear a pending signal
- `next_unblocked()` - Get the next signal to deliver
- `has_pending()` - Check if any unblocked signals are pending

**System Call:**
- `SYS_KILL` (62) - Send signal to process
  - `kill(pid, signal)` - Send signal to specified process

**Example:**

```rust
let mut handler = SignalHandler::new();

// Send a signal
handler.send(Signal::SIGINT);

// Check if pending
if handler.is_pending(Signal::SIGINT) {
    // Handle the signal
    handle_signal(Signal::SIGINT);
    handler.clear(Signal::SIGINT);
}

// Block a signal temporarily
handler.block(Signal::SIGUSR1);
// ... critical section ...
handler.unblock(Signal::SIGUSR1);
```

## Synchronization Primitives

### Semaphore

Counting semaphore for synchronization.

```rust
pub struct Semaphore {
    value: isize,
    waiting_tasks: Vec<TaskId>,
}
```

**Operations:**
- `wait()` - P operation (decrement, block if <= 0)
- `signal()` - V operation (increment, wake waiting task)

**Use Cases:**
- Resource counting
- Producer-consumer synchronization
- Limiting concurrent access

### Task Mutex

Binary lock for mutual exclusion.

```rust
pub struct TaskMutex {
    locked: bool,
    owner: Option<TaskId>,
    waiting_tasks: Vec<TaskId>,
}
```

**Operations:**
- `try_lock()` - Attempt to acquire the lock
- `unlock()` - Release the lock

**Features:**
- Owner tracking
- Waiting queue
- Prevents unlock by non-owner

**Use Cases:**
- Protecting critical sections
- Ensuring atomic operations
- Preventing race conditions

## System Call Interface

### IPC System Calls Summary

| Syscall | Number | Description |
|---------|--------|-------------|
| SYS_PIPE | 22 | Create a pipe |
| SYS_KILL | 62 | Send signal to process |
| SYS_SHMGET | 29 | Get shared memory segment |
| SYS_SHMAT | 30 | Attach shared memory |
| SYS_SHMDT | 67 | Detach shared memory |
| SYS_SHMCTL | 31 | Control shared memory |
| SYS_MSGGET | 68 | Get message queue |
| SYS_MSGSND | 69 | Send message |
| SYS_MSGRCV | 70 | Receive message |

### Error Codes

| Error | Value | Description |
|-------|-------|-------------|
| EINVAL | -22 | Invalid argument |
| EBADF | -9 | Bad file descriptor |
| ENOMEM | -12 | Out of memory |
| ENOSYS | -38 | Function not implemented |
| EFAULT | -14 | Bad address |
| EACCES | -13 | Permission denied |
| EPERM | -1 | Operation not permitted |
| ESRCH | -3 | No such process |

## Implementation Status

### Completed ✅

- ✅ Message Queue data structures
- ✅ Pipe implementation with buffering
- ✅ Shared Memory data structures
- ✅ Signal types and handler
- ✅ Semaphore and Mutex primitives
- ✅ IPC syscall interface
- ✅ Comprehensive unit tests
- ✅ Error handling

### Future Enhancements 🚧

- 🚧 Full file descriptor table integration
- 🚧 Named pipes (FIFOs)
- 🚧 Signal handlers (user-space callbacks)
- 🚧 Advanced semaphore operations
- 🚧 Read/write locks
- 🚧 Condition variables
- 🚧 Message queue priorities
- 🚧 Shared memory permissions

## Testing

The IPC subsystem includes comprehensive unit tests:

```bash
# Run IPC tests
cd kernel/crates/fanga-kernel
cargo test --lib --target x86_64-unknown-linux-gnu ipc
```

**Test Coverage:**
- Message queue operations (send, receive, full queue)
- Pipe operations (read, write, EOF, broken pipe)
- Shared memory attach/detach
- Signal sending, blocking, and delivery
- Signal priority ordering
- Semaphore wait/signal
- Mutex lock/unlock

## Performance Considerations

### IPC Mechanism Performance

| Mechanism | Latency | Throughput | Use Case |
|-----------|---------|------------|----------|
| Shared Memory | Lowest | Highest | Large data transfers |
| Pipes | Low | High | Streaming data |
| Message Queues | Medium | Medium | Discrete messages |
| Signals | Low | N/A | Notifications |

### Recommendations

1. **Shared Memory** - Best for large data transfers or when processes need to work on the same data structure
2. **Pipes** - Best for producer-consumer patterns and streaming data
3. **Message Queues** - Best for discrete messages with FIFO ordering
4. **Signals** - Best for asynchronous notifications and error conditions

## Security Considerations

1. **Permission Checks** - All IPC operations should verify process permissions
2. **Buffer Overflow Protection** - All buffers have maximum size limits
3. **Reference Counting** - Prevents use-after-free in shared memory
4. **Signal Masking** - Critical signals (SIGKILL, SIGSTOP) cannot be blocked
5. **Pointer Validation** - All user-space pointers validated before access

## Examples

### Producer-Consumer with Pipe

```rust
// Producer
let mut pipe = Pipe::new();
pipe.add_writer();
pipe.add_reader();

let data = b"Hello from producer!";
pipe.write(data)?;

// Consumer
let mut buf = [0u8; 1024];
let n = pipe.read(&mut buf)?;
process_data(&buf[..n]);
```

### Shared Memory Communication

```rust
// Process 1: Writer
let mut shm = SharedMemory::new(phys_addr, 4096);
shm.attach(task1)?;
// Write to shared memory...

// Process 2: Reader
shm.attach(task2)?;
// Read from shared memory...
```

### Signal Handling

```rust
let mut handler = SignalHandler::new();

// Install handler
handler.send(Signal::SIGTERM);

// Check for signals periodically
if let Some(signal) = handler.next_unblocked() {
    match signal {
        Signal::SIGTERM => shutdown(),
        Signal::SIGINT => cancel_operation(),
        _ => {}
    }
    handler.clear(signal);
}
```

## References

- [POSIX IPC](https://pubs.opengroup.org/onlinepubs/9699919799/)
- [Linux man pages - IPC](https://man7.org/linux/man-pages/man7/ipc.7.html)
- [Linux man pages - pipe](https://man7.org/linux/man-pages/man2/pipe.2.html)
- [Linux man pages - signal](https://man7.org/linux/man-pages/man7/signal.7.html)
- [OSDev Wiki - IPC](https://wiki.osdev.org/IPC)
