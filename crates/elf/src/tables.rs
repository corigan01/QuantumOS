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

#[repr(C)]
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

impl ElfInitHeader {
    pub fn is_valid(&self) -> bool {
        self.magic == [0x7F, b'E', b'L', b'F']
    }

    pub const fn is_32bit(&self) -> bool {
        self.bits == 1
    }

    pub const fn is_64bit(&self) -> bool {
        self.bits == 2
    }

    pub const fn is_le(&self) -> bool {
        self.endian == 1
    }

    pub const fn is_be(&self) -> bool {
        self.endian == 2
    }

    pub fn arch(&self) -> ArchKind {
        self.arch.into()
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a ElfInitHeader {
    type Error = crate::ElfErrorKind;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.as_ptr() as usize % align_of::<ElfInitHeader>() != 0 {
            return Err(crate::ElfErrorKind::NotAligned);
        }
        if value.len() < size_of::<ElfInitHeader>() {
            return Err(crate::ElfErrorKind::NotEnoughBytes);
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[repr(C)]
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

impl Elf64Header {
    pub fn is_valid(&self) -> bool {
        self.head.magic == [0x7F, b'E', b'L', b'F']
    }

    pub const fn is_32bit(&self) -> bool {
        self.head.bits == 1
    }

    pub const fn is_64bit(&self) -> bool {
        self.head.bits == 2
    }

    pub const fn is_le(&self) -> bool {
        self.head.endian == 1
    }

    pub const fn is_be(&self) -> bool {
        self.head.endian == 2
    }

    pub fn arch(&self) -> ArchKind {
        self.head.arch.into()
    }

    pub const fn program_header_offset(&self) -> u64 {
        self.program_header_offset
    }

    pub const fn program_header_count(&self) -> usize {
        self.program_header_entries as usize
    }

    pub const fn program_header_size(&self) -> usize {
        self.program_header_entry_size as usize
    }

    pub const fn entry_point(&self) -> u64 {
        self.entry_offset
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a Elf64Header {
    type Error = crate::ElfErrorKind;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.as_ptr() as usize % align_of::<Elf64Header>() != 0 {
            return Err(crate::ElfErrorKind::NotAligned);
        }
        if value.len() < size_of::<Elf64Header>() {
            return Err(crate::ElfErrorKind::NotEnoughBytes);
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

#[repr(C)]
#[derive(Debug)]
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

impl Elf32Header {
    pub const fn program_header_offset(&self) -> u32 {
        self.program_header_offset
    }

    pub const fn program_header_count(&self) -> usize {
        self.program_header_entries as usize
    }

    pub const fn program_header_size(&self) -> usize {
        self.program_header_entry_size as usize
    }

    pub const fn entry_point(&self) -> u32 {
        self.entry_offset
    }
}

impl<'a> TryFrom<&'a [u8]> for &'a Elf32Header {
    type Error = crate::ElfErrorKind;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.as_ptr() as usize % align_of::<Elf32Header>() != 0 {
            return Err(crate::ElfErrorKind::NotAligned);
        }
        if value.len() < size_of::<Elf32Header>() {
            return Err(crate::ElfErrorKind::NotEnoughBytes);
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
    Unknown(u16),
}

impl From<u16> for ArchKind {
    fn from(value: u16) -> Self {
        match value {
            0x00 => Self::None,
            0x02 => Self::Sparc,
            0x03 => Self::X86,
            0x08 => Self::Mips,
            0x14 => Self::PowerPC,
            0x28 => Self::Arm,
            0x2a => Self::SuperH,
            0x32 => Self::Ia64,
            0x3e => Self::X64,
            0xb7 => Self::Aarch64,
            0xf3 => Self::RiscV,
            v => Self::Unknown(v),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
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
    type Error = crate::ElfErrorKind;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.as_ptr() as usize % align_of::<ProgramHeader32>() != 0 {
            return Err(crate::ElfErrorKind::NotAligned);
        }
        if value.len() < size_of::<ProgramHeader32>() {
            return Err(crate::ElfErrorKind::NotEnoughBytes);
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

impl ProgramHeader32 {
    pub fn segment_kind(&self) -> SegmentKind {
        self.segment_kind.into()
    }

    pub const fn is_executable(&self) -> bool {
        self.flags & 1 != 0
    }

    pub const fn is_writable(&self) -> bool {
        self.flags & 2 != 0
    }

    pub const fn is_readable(&self) -> bool {
        self.flags & 4 != 0
    }

    pub const fn in_elf_offset(&self) -> usize {
        self.p_offset as usize
    }

    pub const fn expected_vaddr(&self) -> u32 {
        self.p_vaddr
    }

    pub const fn expected_paddr(&self) -> u32 {
        self.p_paddr
    }

    pub const fn in_elf_size(&self) -> usize {
        self.p_filesz as usize
    }

    pub const fn in_mem_size(&self) -> usize {
        self.p_memsz as usize
    }

    pub const fn alignment(&self) -> u32 {
        self.alignment
    }
}

#[repr(C)]
#[derive(Debug)]
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
    type Error = crate::ElfErrorKind;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.as_ptr() as usize % align_of::<ProgramHeader64>() != 0 {
            return Err(crate::ElfErrorKind::NotAligned);
        }
        if value.len() < size_of::<ProgramHeader64>() {
            return Err(crate::ElfErrorKind::NotEnoughBytes);
        }

        Ok(unsafe { &*value.as_ptr().cast() })
    }
}

impl ProgramHeader64 {
    pub fn segment_kind(&self) -> SegmentKind {
        self.segment_kind.into()
    }

    pub const fn is_executable(&self) -> bool {
        self.flags & 1 != 0
    }

    pub const fn is_writable(&self) -> bool {
        self.flags & 2 != 0
    }

    pub const fn is_readable(&self) -> bool {
        self.flags & 4 != 0
    }

    pub const fn in_elf_offset(&self) -> usize {
        self.p_offset as usize
    }

    pub const fn expected_vaddr(&self) -> u64 {
        self.p_vaddr
    }

    pub const fn expected_paddr(&self) -> u64 {
        self.p_paddr
    }

    pub const fn in_elf_size(&self) -> usize {
        self.p_filesz as usize
    }

    pub const fn in_mem_size(&self) -> usize {
        self.p_memsz as usize
    }

    pub const fn alignment(&self) -> u64 {
        self.alignment
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentKind {
    Ignore,
    Load,
    Dynamic,
    Interp,
    Note,
    Unknown(u32),
}

impl From<u32> for SegmentKind {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Ignore,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Interp,
            4 => Self::Note,
            v => Self::Unknown(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ElfHeader<'a> {
    Header64(&'a Elf64Header),
    Header32(&'a Elf32Header),
}

#[derive(Debug, Clone, Copy)]
pub enum ElfProgramHeaders<'a> {
    ProgHeader64(&'a [ProgramHeader64]),
    ProgHeader32(&'a [ProgramHeader32]),
}

#[derive(Debug, Clone, Copy)]
pub struct ElfProgramHeaderIter<'a> {
    headers: ElfProgramHeaders<'a>,
    index: usize,
}

impl<'a> ElfProgramHeaders<'a> {
    pub fn iter(&self) -> ElfProgramHeaderIter<'a> {
        ElfProgramHeaderIter {
            headers: *self,
            index: 0,
        }
    }
}

impl<'a> Iterator for ElfProgramHeaderIter<'a> {
    type Item = ElfGenProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        match self.headers {
            ElfProgramHeaders::ProgHeader64(h64) => {
                let gen_header = h64
                    .get(self.index)
                    .map(|h64_el| ElfGenProgramHeader::from(h64_el))?;
                self.index += 1;

                Some(gen_header)
            }
            ElfProgramHeaders::ProgHeader32(h32) => {
                let gen_header = h32
                    .get(self.index)
                    .map(|h32_el| ElfGenProgramHeader::from(h32_el))?;
                self.index += 1;

                Some(gen_header)
            }
        }
    }
}

#[derive(Debug)]
pub struct ElfGenProgramHeader {
    bits: u8,
    segment_kind: u32,
    flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    alignment: u64,
}

impl From<&ProgramHeader64> for ElfGenProgramHeader {
    fn from(value: &ProgramHeader64) -> Self {
        Self {
            bits: 2,
            segment_kind: value.segment_kind,
            flags: value.flags,
            p_offset: value.p_offset,
            p_vaddr: value.p_vaddr,
            p_paddr: value.p_paddr,
            p_filesz: value.p_filesz,
            p_memsz: value.p_memsz,
            alignment: value.alignment,
        }
    }
}

impl From<&ProgramHeader32> for ElfGenProgramHeader {
    fn from(value: &ProgramHeader32) -> Self {
        Self {
            bits: 1,
            segment_kind: value.segment_kind,
            flags: value.flags,
            p_offset: value.p_offset as u64,
            p_vaddr: value.p_vaddr as u64,
            p_paddr: value.p_paddr as u64,
            p_filesz: value.p_filesz as u64,
            p_memsz: value.p_memsz as u64,
            alignment: value.alignment as u64,
        }
    }
}

impl ElfGenProgramHeader {
    pub const fn is_64bit(&self) -> bool {
        self.bits == 2
    }

    pub const fn is_32bit(&self) -> bool {
        self.bits == 1
    }

    pub fn segment_kind(&self) -> SegmentKind {
        self.segment_kind.into()
    }

    pub const fn is_executable(&self) -> bool {
        self.flags & 1 != 0
    }

    pub const fn is_writable(&self) -> bool {
        self.flags & 2 != 0
    }

    pub const fn is_readable(&self) -> bool {
        self.flags & 4 != 0
    }

    pub const fn in_elf_offset(&self) -> usize {
        self.p_offset as usize
    }

    pub const fn expected_vaddr(&self) -> u64 {
        self.p_vaddr
    }

    pub const fn expected_paddr(&self) -> u64 {
        self.p_paddr
    }

    pub const fn in_elf_size(&self) -> usize {
        self.p_filesz as usize
    }

    pub const fn in_mem_size(&self) -> usize {
        self.p_memsz as usize
    }

    pub const fn alignment(&self) -> u64 {
        self.alignment
    }
}
