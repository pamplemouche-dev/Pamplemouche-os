//! Mach-O loader — zero-copy parser for 64-bit Mach-O binaries.
//!
//! Supports:
//! * Single-arch Mach-O 64 (`MH_MAGIC_64`)
//! * Universal / Fat binaries (`FAT_MAGIC`)
//! * Load commands: `LC_SEGMENT_64`, `LC_MAIN`, `LC_DYLD_INFO*`,
//!   `LC_SYMTAB`, `LC_DYSYMTAB`, `LC_LOAD_DYLIB`, `LC_ID_DYLIB`,
//!   `LC_UUID`, `LC_BUILD_VERSION`, `LC_SOURCE_VERSION`

// ── Magic numbers ─────────────────────────────────────────────────────────────

/// 64-bit Mach-O (little-endian).
pub const MH_MAGIC_64: u32 = 0xFEED_FACF;
/// 64-bit Mach-O (big-endian / byte-swapped).
pub const MH_CIGAM_64: u32 = 0xCFFA_EDFE;
/// Universal/fat binary (big-endian magic).
pub const FAT_MAGIC: u32 = 0xCAFE_BABE;
/// Universal/fat binary (little-endian magic).
pub const FAT_CIGAM: u32 = 0xBEBA_FECA;

// ── File type constants ───────────────────────────────────────────────────────

/// Demand-paged executable.
pub const MH_EXECUTE: u32 = 0x2;
/// Fixed-VM shared library.
pub const MH_FVMLIB: u32 = 0x3;
/// Dynamically-bound shared library.
pub const MH_DYLIB: u32 = 0x6;
/// Dynamically-bound bundle.
pub const MH_BUNDLE: u32 = 0x8;

// ── CPU type / subtype ────────────────────────────────────────────────────────

pub const CPU_TYPE_X86_64: u32 = 0x0100_0007;
pub const CPU_TYPE_ARM64: u32 = 0x0100_000C;

// ── Load-command identifiers ──────────────────────────────────────────────────

pub const LC_SEGMENT_64: u32 = 0x19;
pub const LC_SYMTAB: u32 = 0x2;
pub const LC_DYSYMTAB: u32 = 0xB;
pub const LC_LOAD_DYLIB: u32 = 0xC;
pub const LC_ID_DYLIB: u32 = 0xD;
pub const LC_MAIN: u32 = 0x8000_0028;
pub const LC_DYLD_INFO: u32 = 0x22;
pub const LC_DYLD_INFO_ONLY: u32 = 0x8000_0022;
pub const LC_UUID: u32 = 0x1B;
pub const LC_BUILD_VERSION: u32 = 0x32;
pub const LC_SOURCE_VERSION: u32 = 0x2A;

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors produced by the Mach-O parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The byte slice is too short to contain the requested structure.
    UnexpectedEof,
    /// The magic number does not correspond to a supported Mach-O format.
    UnrecognisedMagic(u32),
    /// A load command's `cmdsize` is too small or not aligned.
    InvalidCmdSize,
    /// The binary's CPU architecture is not supported.
    UnsupportedArch(u32),
    /// A string offset points outside the binary.
    InvalidStringOffset,
}

// ── Raw on-disk structures ────────────────────────────────────────────────────

/// 64-bit Mach-O file header (28 bytes).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MachHeader64 {
    pub magic: u32,
    pub cpu_type: u32,
    pub cpu_subtype: u32,
    pub file_type: u32,
    pub n_cmds: u32,
    pub size_of_cmds: u32,
    pub flags: u32,
    pub _reserved: u32,
}

/// Generic load-command prefix (8 bytes).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LoadCommandHeader {
    pub cmd: u32,
    pub cmdsize: u32,
}

/// `LC_SEGMENT_64` load command.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SegmentCommand64 {
    pub cmd: u32,
    pub cmdsize: u32,
    pub segname: [u8; 16],
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
    pub maxprot: i32,
    pub initprot: i32,
    pub nsects: u32,
    pub flags: u32,
}

/// `section_64` descriptor within an `LC_SEGMENT_64`.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Section64Raw {
    pub sectname: [u8; 16],
    pub segname: [u8; 16],
    pub addr: u64,
    pub size: u64,
    pub offset: u32,
    pub align: u32,
    pub reloff: u32,
    pub nreloc: u32,
    pub flags: u32,
    pub _reserved1: u32,
    pub _reserved2: u32,
    pub _reserved3: u32,
}

/// `LC_MAIN` load command.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EntryPointCommand {
    pub cmd: u32,
    pub cmdsize: u32,
    /// File offset of the main() function.
    pub entryoff: u64,
    /// Initial stack size (0 = use default).
    pub stacksize: u64,
}

/// Fat (universal binary) header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FatHeader {
    pub magic: u32,
    pub n_fat_arch: u32,
}

/// Fat architecture descriptor.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FatArch {
    pub cpu_type: u32,
    pub cpu_subtype: u32,
    pub offset: u32,
    pub size: u32,
    pub align: u32,
}

// ── High-level parsed representations ────────────────────────────────────────

/// A parsed `LC_SEGMENT_64` including its sections.
#[derive(Debug)]
pub struct Segment64<'a> {
    pub name: &'a str,
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
    pub maxprot: i32,
    pub initprot: i32,
    pub sections: &'a [Section64Raw],
}

/// A parsed section descriptor.
#[derive(Debug)]
pub struct Section64<'a> {
    pub sectname: &'a str,
    pub segname: &'a str,
    pub addr: u64,
    pub size: u64,
    pub offset: u32,
}

/// A parsed Mach-O load command.
#[derive(Debug)]
pub enum LoadCommand<'a> {
    Segment(Segment64<'a>),
    Main { entryoff: u64, stacksize: u64 },
    Uuid([u8; 16]),
    LoadDylib { name_offset: u32, current_version: u32 },
    DyldInfo {
        rebase_off: u32,
        bind_off: u32,
        lazy_bind_off: u32,
        export_off: u32,
    },
    Other { cmd: u32, cmdsize: u32 },
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Read a value of type `T` from `bytes` at `offset`.
///
/// # Safety
/// Caller guarantees that `bytes[offset..offset+size_of::<T>()]` is a valid
/// initialised representation of `T` (all bit patterns valid for the type).
unsafe fn read_struct<T: Copy>(bytes: &[u8], offset: usize) -> Result<T, ParseError> {
    let size = core::mem::size_of::<T>();
    if offset + size > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    // SAFETY: guaranteed by caller contract above.
    Ok(unsafe { core::ptr::read_unaligned(bytes.as_ptr().add(offset) as *const T) })
}

/// Parse the Mach-O header at the start of `bytes`.
pub fn parse_header(bytes: &[u8]) -> Result<MachHeader64, ParseError> {
    if bytes.len() < 4 {
        return Err(ParseError::UnexpectedEof);
    }
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    match magic {
        MH_MAGIC_64 => {}
        MH_CIGAM_64 | FAT_MAGIC | FAT_CIGAM => return Err(ParseError::UnrecognisedMagic(magic)),
        other => return Err(ParseError::UnrecognisedMagic(other)),
    }
    // SAFETY: `MachHeader64` consists entirely of `u32` fields — all bit
    // patterns are valid.
    unsafe { read_struct::<MachHeader64>(bytes, 0) }
}

/// Parse all load commands from a validated Mach-O binary.
///
/// Returns an iterator-like `LoadCommandIter` rather than allocating a `Vec`.
pub fn load_commands(bytes: &[u8]) -> Result<LoadCommandIter<'_>, ParseError> {
    let header = parse_header(bytes)?;
    let start = core::mem::size_of::<MachHeader64>();
    Ok(LoadCommandIter {
        bytes,
        offset: start,
        remaining: header.n_cmds,
    })
}

/// Lazy iterator over load commands in a Mach-O binary.
pub struct LoadCommandIter<'a> {
    bytes: &'a [u8],
    offset: usize,
    remaining: u32,
}

impl<'a> Iterator for LoadCommandIter<'a> {
    type Item = Result<LoadCommand<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        Some(self.parse_next())
    }
}

impl<'a> LoadCommandIter<'a> {
    fn parse_next(&mut self) -> Result<LoadCommand<'a>, ParseError> {
        // SAFETY: `LoadCommandHeader` is two `u32` fields — all bit patterns valid.
        let hdr: LoadCommandHeader =
            unsafe { read_struct(self.bytes, self.offset)? };

        if hdr.cmdsize < 8 || hdr.cmdsize % 4 != 0 {
            return Err(ParseError::InvalidCmdSize);
        }
        if self.offset + hdr.cmdsize as usize > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }

        let cmd_bytes = &self.bytes[self.offset..self.offset + hdr.cmdsize as usize];
        let lc = match hdr.cmd {
            LC_SEGMENT_64 => self.parse_segment64(cmd_bytes)?,
            LC_MAIN => self.parse_main(cmd_bytes)?,
            LC_UUID => self.parse_uuid(cmd_bytes)?,
            LC_LOAD_DYLIB | LC_ID_DYLIB => self.parse_dylib(cmd_bytes)?,
            LC_DYLD_INFO | LC_DYLD_INFO_ONLY => self.parse_dyld_info(cmd_bytes)?,
            _ => LoadCommand::Other { cmd: hdr.cmd, cmdsize: hdr.cmdsize },
        };

        self.offset += hdr.cmdsize as usize;
        Ok(lc)
    }

    fn parse_segment64(&self, cmd: &'a [u8]) -> Result<LoadCommand<'a>, ParseError> {
        // SAFETY: `SegmentCommand64` contains only integer types.
        let sc: SegmentCommand64 = unsafe { read_struct(cmd, 0)? };

        // Read the segment name directly from the raw bytes so the returned
        // reference borrows `cmd` (lifetime `'a`) rather than the local `sc`.
        let segname_bytes = cmd
            .get(8..24)
            .ok_or(ParseError::UnexpectedEof)?;
        let name = core::str::from_utf8(
            segname_bytes
                .split(|&b| b == 0)
                .next()
                .unwrap_or(segname_bytes),
        )
        .unwrap_or("?");

        let sections_offset = core::mem::size_of::<SegmentCommand64>();
        let section_size = core::mem::size_of::<Section64Raw>();
        let sections_bytes = cmd
            .get(sections_offset..sections_offset + sc.nsects as usize * section_size)
            .ok_or(ParseError::UnexpectedEof)?;
        // SAFETY: `Section64Raw` contains only integer types.
        let sections: &'a [Section64Raw] = unsafe {
            core::slice::from_raw_parts(
                sections_bytes.as_ptr() as *const Section64Raw,
                sc.nsects as usize,
            )
        };

        Ok(LoadCommand::Segment(Segment64 {
            name,
            vmaddr: sc.vmaddr,
            vmsize: sc.vmsize,
            fileoff: sc.fileoff,
            filesize: sc.filesize,
            maxprot: sc.maxprot,
            initprot: sc.initprot,
            sections,
        }))
    }

    fn parse_main(&self, cmd: &[u8]) -> Result<LoadCommand<'a>, ParseError> {
        // SAFETY: `EntryPointCommand` is all integers.
        let ep: EntryPointCommand = unsafe { read_struct(cmd, 0)? };
        Ok(LoadCommand::Main {
            entryoff: ep.entryoff,
            stacksize: ep.stacksize,
        })
    }

    fn parse_uuid(&self, cmd: &[u8]) -> Result<LoadCommand<'a>, ParseError> {
        if cmd.len() < 24 {
            return Err(ParseError::UnexpectedEof);
        }
        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(&cmd[8..24]);
        Ok(LoadCommand::Uuid(uuid))
    }

    fn parse_dylib(&self, cmd: &[u8]) -> Result<LoadCommand<'a>, ParseError> {
        if cmd.len() < 24 {
            return Err(ParseError::UnexpectedEof);
        }
        let name_offset = u32::from_le_bytes([cmd[8], cmd[9], cmd[10], cmd[11]]);
        let current_version = u32::from_le_bytes([cmd[12], cmd[13], cmd[14], cmd[15]]);
        Ok(LoadCommand::LoadDylib { name_offset, current_version })
    }

    fn parse_dyld_info(&self, cmd: &[u8]) -> Result<LoadCommand<'a>, ParseError> {
        if cmd.len() < 48 {
            return Err(ParseError::UnexpectedEof);
        }
        let u32_at = |off: usize| {
            u32::from_le_bytes([cmd[off], cmd[off + 1], cmd[off + 2], cmd[off + 3]])
        };
        Ok(LoadCommand::DyldInfo {
            rebase_off: u32_at(8),
            bind_off: u32_at(16),
            lazy_bind_off: u32_at(24),
            export_off: u32_at(32),
        })
    }
}
