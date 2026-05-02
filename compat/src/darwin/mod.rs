//! Darwin/XNU syscall compatibility layer.
//!
//! Translates Darwin BSD syscall numbers (as used by macOS binaries) into
//! Pamplemouche-OS kernel requests.
//!
//! This module is intentionally complete with respect to the syscall *table*
//! so that binaries can be introspected; individual syscalls are wired to
//! actual kernel implementations as those subsystems are built.

pub mod syscalls;

pub use syscalls::{darwin_syscall, DarwinSyscallResult, SyscallNumber};
