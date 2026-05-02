//! Pamplemouche-OS compatibility layer.
//!
//! Provides:
//! - [`macho`] — Mach-O binary parser (header, load commands, segments).
//! - [`darwin`] — Darwin/XNU syscall translation table.
//!
//! # Design
//! This crate is `#![no_std]` so it can be linked directly into the kernel or
//! into a thin user-space server.  All heap-less parsing is done with raw byte
//! slices; the loader that actually maps segments into virtual memory lives in
//! the kernel and calls into this crate.

#![no_std]

pub mod darwin;
pub mod macho;
