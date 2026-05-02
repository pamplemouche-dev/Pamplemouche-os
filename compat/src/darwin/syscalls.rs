//! Darwin/XNU BSD syscall table and dispatch.
//!
//! Syscall numbers are taken from Apple's open-source XNU kernel
//! (`bsd/kern/syscalls.master`) and are stable across macOS versions for
//! compatibility.
//!
//! # Syscall calling convention (x86-64 macOS / Darwin ABI)
//! `rax` = syscall number (BSD class: `0x2000000 | N`)
//! `rdi, rsi, rdx, r10, r8, r9` = arguments
//! `rcx` destroyed, `r11` destroyed
//! Returns: `rax` (value), `rdx` (2nd return), `CF` set on error.

// ── Syscall numbers ────────────────────────────────────────────────────────────

/// Darwin BSD syscall number.  The `0x2000000` Unix-class offset is *not*
/// included — callers strip it before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum SyscallNumber {
    // -- Process --
    Exit = 1,
    Fork = 2,
    Read = 3,
    Write = 4,
    Open = 5,
    Close = 6,
    Wait4 = 7,
    // (8 reserved)
    Link = 9,
    Unlink = 10,
    // (11 reserved)
    Chdir = 12,
    Fchdir = 13,
    Mknod = 14,
    Chmod = 15,
    Chown = 16,
    // (17 reserved)
    Getfsstat = 18,
    // (19 reserved)
    Getpid = 20,
    // ...
    Setuid = 23,
    Getuid = 24,
    Geteuid = 25,
    // ...
    Recvmsg = 27,
    Sendmsg = 28,
    Recvfrom = 29,
    Accept = 30,
    Getpeername = 31,
    Getsockname = 32,
    // ...
    Kill = 37,
    // ...
    Getppid = 39,
    // ...
    Dup = 41,
    Pipe = 42,
    Getegid = 43,
    // ...
    Sigaction = 46,
    Getgid = 47,
    Sigprocmask = 48,
    Getlogin = 49,
    Setlogin = 50,
    Acct = 51,
    Sigpending = 52,
    Sigaltstack = 53,
    Ioctl = 54,
    // ...
    Execve = 59,
    Umask = 60,
    Chroot = 61,
    // ...
    Munmap = 73,
    Mprotect = 74,
    Madvise = 75,
    // ...
    Mincore = 78,
    Getgroups = 79,
    Setgroups = 80,
    Getpgrp = 81,
    Setpgid = 82,
    Setitimer = 83,
    // ...
    Swapon = 85,
    Getitimer = 86,
    // ...
    Getdtablesize = 89,
    Dup2 = 90,
    // ...
    Fcntl = 92,
    Select = 93,
    // ...
    Fsync = 95,
    Setpriority = 96,
    Socket = 97,
    Connect = 98,
    // ...
    Getpriority = 100,
    // ...
    Bind = 104,
    Setsockopt = 105,
    Listen = 106,
    // ...
    Sigsuspend = 111,
    // ...
    Gettimeofday = 116,
    Getrusage = 117,
    Getsockopt = 118,
    // ...
    Readv = 120,
    Writev = 121,
    Settimeofday = 122,
    Fchown = 123,
    Fchmod = 124,
    // ...
    Setreuid = 126,
    Setregid = 127,
    Rename = 128,
    // ...
    Flock = 131,
    Mkfifo = 132,
    Sendto = 133,
    Shutdown = 134,
    Socketpair = 135,
    Mkdir = 136,
    Rmdir = 137,
    Utimes = 138,
    Futimes = 139,
    Adjtime = 140,
    // ...
    Getpagesize = 190,
    // ...
    Mmap = 197,
    // ...
    Lseek = 199,
    Truncate = 200,
    Ftruncate = 201,
    // ...
    Sysctl = 202,
    Mlock = 203,
    Munlock = 204,
    Undelete = 205,
    // ...
    Issetugid = 327,
    // ...
    Posix_spawn = 244,
    // ...
    // Catch-all for unrecognised numbers.
    Unknown = 0xFFFF_FFFF,
}

impl SyscallNumber {
    /// Convert a raw `u32` syscall number (BSD class stripped) to the enum.
    pub fn from_raw(n: u32) -> Self {
        // A lookup table would be more efficient but requires allocation or a
        // large match.  This is fine for the initial implementation.
        match n {
            1 => Self::Exit,
            2 => Self::Fork,
            3 => Self::Read,
            4 => Self::Write,
            5 => Self::Open,
            6 => Self::Close,
            7 => Self::Wait4,
            9 => Self::Link,
            10 => Self::Unlink,
            12 => Self::Chdir,
            13 => Self::Fchdir,
            14 => Self::Mknod,
            15 => Self::Chmod,
            16 => Self::Chown,
            20 => Self::Getpid,
            23 => Self::Setuid,
            24 => Self::Getuid,
            25 => Self::Geteuid,
            27 => Self::Recvmsg,
            28 => Self::Sendmsg,
            29 => Self::Recvfrom,
            30 => Self::Accept,
            31 => Self::Getpeername,
            32 => Self::Getsockname,
            37 => Self::Kill,
            39 => Self::Getppid,
            41 => Self::Dup,
            42 => Self::Pipe,
            43 => Self::Getegid,
            46 => Self::Sigaction,
            47 => Self::Getgid,
            48 => Self::Sigprocmask,
            49 => Self::Getlogin,
            50 => Self::Setlogin,
            51 => Self::Acct,
            52 => Self::Sigpending,
            53 => Self::Sigaltstack,
            54 => Self::Ioctl,
            59 => Self::Execve,
            60 => Self::Umask,
            61 => Self::Chroot,
            73 => Self::Munmap,
            74 => Self::Mprotect,
            75 => Self::Madvise,
            78 => Self::Mincore,
            79 => Self::Getgroups,
            80 => Self::Setgroups,
            81 => Self::Getpgrp,
            82 => Self::Setpgid,
            83 => Self::Setitimer,
            85 => Self::Swapon,
            86 => Self::Getitimer,
            89 => Self::Getdtablesize,
            90 => Self::Dup2,
            92 => Self::Fcntl,
            93 => Self::Select,
            95 => Self::Fsync,
            96 => Self::Setpriority,
            97 => Self::Socket,
            98 => Self::Connect,
            100 => Self::Getpriority,
            104 => Self::Bind,
            105 => Self::Setsockopt,
            106 => Self::Listen,
            111 => Self::Sigsuspend,
            116 => Self::Gettimeofday,
            117 => Self::Getrusage,
            118 => Self::Getsockopt,
            120 => Self::Readv,
            121 => Self::Writev,
            122 => Self::Settimeofday,
            123 => Self::Fchown,
            124 => Self::Fchmod,
            126 => Self::Setreuid,
            127 => Self::Setregid,
            128 => Self::Rename,
            131 => Self::Flock,
            132 => Self::Mkfifo,
            133 => Self::Sendto,
            134 => Self::Shutdown,
            135 => Self::Socketpair,
            136 => Self::Mkdir,
            137 => Self::Rmdir,
            138 => Self::Utimes,
            139 => Self::Futimes,
            140 => Self::Adjtime,
            190 => Self::Getpagesize,
            197 => Self::Mmap,
            199 => Self::Lseek,
            200 => Self::Truncate,
            201 => Self::Ftruncate,
            202 => Self::Sysctl,
            203 => Self::Mlock,
            204 => Self::Munlock,
            205 => Self::Undelete,
            244 => Self::Posix_spawn,
            327 => Self::Issetugid,
            _ => Self::Unknown,
        }
    }
}

// ── Result type ───────────────────────────────────────────────────────────────

/// The result of a Darwin syscall dispatch.
#[derive(Debug, Clone, Copy)]
pub enum DarwinSyscallResult {
    /// Success: primary return value and optional secondary return value.
    Ok(u64, u64),
    /// Error: Darwin `errno` value.
    Err(u32),
    /// Syscall is not yet implemented.
    Unimplemented,
}

// ── Errno constants (POSIX / Darwin) ─────────────────────────────────────────

pub const EPERM: u32 = 1;
pub const ENOENT: u32 = 2;
pub const ESRCH: u32 = 3;
pub const EINTR: u32 = 4;
pub const EIO: u32 = 5;
pub const ENXIO: u32 = 6;
pub const E2BIG: u32 = 7;
pub const ENOEXEC: u32 = 8;
pub const EBADF: u32 = 9;
pub const ECHILD: u32 = 10;
pub const EDEADLK: u32 = 11;
pub const ENOMEM: u32 = 12;
pub const EACCES: u32 = 13;
pub const EFAULT: u32 = 14;
pub const ENOTBLK: u32 = 15;
pub const EBUSY: u32 = 16;
pub const EEXIST: u32 = 17;
pub const EXDEV: u32 = 18;
pub const ENODEV: u32 = 19;
pub const ENOTDIR: u32 = 20;
pub const EISDIR: u32 = 21;
pub const EINVAL: u32 = 22;
pub const ENFILE: u32 = 23;
pub const EMFILE: u32 = 24;
pub const ENOTTY: u32 = 25;
pub const ETXTBSY: u32 = 26;
pub const EFBIG: u32 = 27;
pub const ENOSPC: u32 = 28;
pub const ESPIPE: u32 = 29;
pub const EROFS: u32 = 30;
pub const EMLINK: u32 = 31;
pub const EPIPE: u32 = 32;
pub const ENOSYS: u32 = 78;

// ── Dispatcher ────────────────────────────────────────────────────────────────

/// Dispatch a Darwin BSD syscall.
///
/// `nr_raw` is the raw `rax` value with the BSD class bits (`0x2000000`)
/// already stripped.  `args` are `[rdi, rsi, rdx, r10, r8, r9]`.
///
/// Returns a [`DarwinSyscallResult`] which the low-level syscall stub
/// translates back into registers and the carry flag.
pub fn darwin_syscall(nr_raw: u32, args: [u64; 6]) -> DarwinSyscallResult {
    let nr = SyscallNumber::from_raw(nr_raw);
    match nr {
        SyscallNumber::Exit => {
            // args[0] = exit code; in the full kernel this tears down the process.
            // For now, return Ok so the stub can spin in the idle loop.
            DarwinSyscallResult::Ok(args[0], 0)
        }
        SyscallNumber::Getpid => {
            // Return a placeholder PID; the real value comes from the process table.
            DarwinSyscallResult::Ok(1, 0)
        }
        SyscallNumber::Getuid | SyscallNumber::Geteuid => {
            // Running as root in the compatibility layer.
            DarwinSyscallResult::Ok(0, 0)
        }
        SyscallNumber::Getgid | SyscallNumber::Getegid => {
            DarwinSyscallResult::Ok(0, 0)
        }
        SyscallNumber::Issetugid => {
            // Not a set-uid / set-gid binary.
            DarwinSyscallResult::Ok(0, 0)
        }
        SyscallNumber::Getpagesize => {
            DarwinSyscallResult::Ok(4096, 0)
        }
        SyscallNumber::Read => {
            // fd=args[0], buf=args[1], count=args[2]
            // Stub: real I/O will be wired to VFS.
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Write => {
            // fd=args[0], buf=args[1], count=args[2]
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Mmap => {
            // addr=args[0], len=args[1], prot=args[2], flags=args[3],
            // fd=args[4], offset=args[5]
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Munmap => {
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Mprotect => {
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Open => {
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Close => {
            DarwinSyscallResult::Err(ENOSYS)
        }
        SyscallNumber::Unknown => {
            DarwinSyscallResult::Err(ENOSYS)
        }
        _ => DarwinSyscallResult::Unimplemented,
    }
}
