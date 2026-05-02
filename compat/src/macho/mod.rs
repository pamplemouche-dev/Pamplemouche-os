//! Mach-O binary format parser.
//!
//! Implements a zero-copy parser for the 64-bit Mach-O object file format
//! used by macOS executables, dylibs, and dyld shared caches.
//!
//! Reference: `<mach-o/loader.h>` from the XNU source tree (open-source,
//! <https://github.com/apple-oss-distributions/xnu>).

pub mod loader;

pub use loader::{
    FatHeader, LoadCommand, MachHeader64, ParseError, Section64, Segment64,
};
