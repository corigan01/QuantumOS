/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
    Part of the Quantum OS Project

Copyright 2024 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct ElfInitHeader {
    magic: [u8; 4],
    bits: u8,
    endian: u8,
    header_version: u8,
    os_abi: u8,
    padding: [u8; 8],
    kind: u16,
    arch: u16,
    elf_version: u32,
}

impl<'a> TryFrom<&'a [u8]> for &'a ElfInitHeader {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<ElfInitHeader>() {
            return Err(());
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct Elf64Header {
    head: ElfInitHeader,
    entry_offset: u64,
    program_header_offset: u64,
    section_header_offset: u64,
    flags: u32,
    elf_header_size: u16,
    program_header_entry_size: u16,
    program_header_entries: u16,
    section_header_entry_size: u16,
    section_header_entries: u16,
    string_table_offset: u16,
}

impl<'a> TryFrom<&'a [u8]> for &'a Elf64Header {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Elf64Header>() {
            return Err(());
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[repr(C, packed)]
pub struct Elf32Header {
    head: ElfInitHeader,
    entry_offset: u32,
    program_header_offset: u32,
    section_header_offset: u32,
    flags: u32,
    elf_header_size: u16,
    program_header_entry_size: u16,
    program_header_entries: u16,
    section_header_entry_size: u16,
    section_header_entries: u16,
    string_table_offset: u16,
}

impl<'a> TryFrom<&'a [u8]> for &'a Elf32Header {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<Elf32Header>() {
            return Err(());
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchKind {
    None,
    Sparc,
    X86,
    Mips,
    PowerPC,
    Arm,
    SuperH,
    Ia64,
    X64,
    Aarch64,
    RiscV,
}

impl TryFrom<u16> for ArchKind {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::None),
            0x02 => Ok(Self::Sparc),
            0x03 => Ok(Self::X86),
            0x08 => Ok(Self::Mips),
            0x14 => Ok(Self::PowerPC),
            0x28 => Ok(Self::Arm),
            0x2a => Ok(Self::SuperH),
            0x32 => Ok(Self::Ia64),
            0x3e => Ok(Self::X64),
            0xb7 => Ok(Self::Aarch64),
            0xf3 => Ok(Self::RiscV),
            _ => Err(()),
        }
    }
}

#[repr(C, packed)]
pub struct ProgramHeader32 {
    segment_kind: u32,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    flags: u32,
    alignment: u32,
}

impl<'a> TryFrom<&'a [u8]> for &'a ProgramHeader32 {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<ProgramHeader32>() {
            return Err(());
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[repr(C, packed)]
pub struct ProgramHeader64 {
    segment_kind: u32,
    flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    alignment: u64,
}

impl<'a> TryFrom<&'a [u8]> for &'a ProgramHeader64 {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < size_of::<ProgramHeader64>() {
            return Err(());
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SegmentKind {
    Ignore,
    Load,
    Dynamic,
    Interp,
    Note,
}

impl TryFrom<u32> for SegmentKind {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ignore),
            1 => Ok(Self::Load),
            2 => Ok(Self::Dynamic),
            3 => Ok(Self::Interp),
            4 => Ok(Self::Note),
            _ => Err(()),
        }
    }
}
