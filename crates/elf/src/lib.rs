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

#![no_std]

pub mod tables;

#[derive(Clone, Copy, Debug)]
pub enum ElfErrorKind {
    NotEnoughBytes,
    NotAligned,
    IncorrectBitMode,
    Invalid,
}

pub type Result<T> = core::result::Result<T, ElfErrorKind>;

// TODO: Add 'loader' trait for calling code when a section needs to be loaded

pub struct Elf<'a> {
    elf_file: &'a [u8],
}

impl<'a> Elf<'a> {
    pub fn new(elf_file: &'a [u8]) -> Self {
        Self { elf_file }
    }

    pub fn load_into<F>(&self, mut loader_fn: F) -> Result<*const u8>
    where
        F: FnMut(&tables::ElfGenProgramHeader) -> Option<&'static mut [u8]>,
    {
        match self.program_headers()? {
            tables::ElfProgramHeaders::ProgHeader64(header) => header
                .iter()
                .map(|h| tables::ElfGenProgramHeader::from(h))
                .try_for_each(|h| {
                    if let Some(mem_buffer) = loader_fn(&h) {
                        let elf_buffer = self
                            .elf_file
                            .get(h.in_elf_offset()..h.in_elf_offset() + h.in_elf_size())
                            .ok_or(ElfErrorKind::NotEnoughBytes)?;

                        if h.in_elf_size() > mem_buffer.len() {
                            return Err(ElfErrorKind::Invalid);
                        }

                        mem_buffer[..h.in_elf_size()].copy_from_slice(elf_buffer);
                    }

                    Ok(())
                })?,
            tables::ElfProgramHeaders::ProgHeader32(header) => header
                .iter()
                .map(|h| tables::ElfGenProgramHeader::from(h))
                .try_for_each(|h| {
                    if let Some(mem_buffer) = loader_fn(&h) {
                        let elf_buffer = self
                            .elf_file
                            .get(h.in_elf_offset()..h.in_elf_offset() + h.in_elf_size())
                            .ok_or(ElfErrorKind::NotEnoughBytes)?;

                        if h.in_elf_size() > mem_buffer.len() {
                            return Err(ElfErrorKind::Invalid);
                        }

                        mem_buffer[..h.in_elf_size()].copy_from_slice(elf_buffer);
                    }

                    Ok(())
                })?,
        }

        self.entry_point()
    }

    pub fn exe_size(&self) -> Result<usize> {
        let mut lowest_addr = u64::MAX;
        let mut highest_addr = 0;

        match self.program_headers()? {
            tables::ElfProgramHeaders::ProgHeader64(header) => header
                .iter()
                .map(|h| tables::ElfGenProgramHeader::from(h))
                .filter(|h| h.segment_kind() == tables::SegmentKind::Load)
                .for_each(|h| {
                    lowest_addr = lowest_addr.min(h.expected_vaddr());
                    highest_addr = highest_addr.max(h.expected_vaddr() + h.in_mem_size() as u64);
                }),
            tables::ElfProgramHeaders::ProgHeader32(header) => header
                .iter()
                .map(|h| tables::ElfGenProgramHeader::from(h))
                .filter(|h| h.segment_kind() == tables::SegmentKind::Load)
                .for_each(|h| {
                    lowest_addr = lowest_addr.min(h.expected_vaddr());
                    highest_addr = highest_addr.max(h.expected_vaddr() + h.in_mem_size() as u64);
                }),
        }

        if lowest_addr >= highest_addr {
            Ok(0)
        } else {
            Ok((highest_addr - lowest_addr) as usize)
        }
    }

    pub fn entry_point(&self) -> Result<*const u8> {
        Ok(match self.header()? {
            tables::ElfHeader::Header64(h) => h.entry_point() as *const u8,
            tables::ElfHeader::Header32(h) => h.entry_point() as *const u8,
        })
    }

    pub fn header(&self) -> Result<tables::ElfHeader<'a>> {
        let pre_header = self
            .elf_file
            .try_into()
            .and_then(|h: &tables::ElfInitHeader| {
                if h.is_valid() {
                    Ok(h)
                } else {
                    Err(ElfErrorKind::Invalid)
                }
            })?;

        if pre_header.is_64bit() {
            let header: &tables::Elf64Header = self.elf_file.try_into()?;
            Ok(tables::ElfHeader::Header64(header))
        } else {
            let header: &tables::Elf32Header = self.elf_file.try_into()?;
            Ok(tables::ElfHeader::Header32(header))
        }
    }

    pub fn program_headers(&self) -> Result<tables::ElfProgramHeaders<'a>> {
        let header = self.header()?;

        let (offset, n_entries, entry_size) = match header {
            tables::ElfHeader::Header64(header) => (
                header.program_header_offset() as usize,
                header.program_header_count(),
                header.program_header_size(),
            ),
            tables::ElfHeader::Header32(header) => (
                header.program_header_offset() as usize,
                header.program_header_count(),
                header.program_header_size(),
            ),
        };

        let program_header_slice = &self.elf_file[offset..(offset + (n_entries * entry_size))];

        match header {
            tables::ElfHeader::Header64(_) => Ok(tables::ElfProgramHeaders::ProgHeader64(unsafe {
                core::slice::from_raw_parts(program_header_slice.as_ptr().cast(), n_entries)
            })),
            tables::ElfHeader::Header32(_) => Ok(tables::ElfProgramHeaders::ProgHeader32(unsafe {
                core::slice::from_raw_parts(program_header_slice.as_ptr().cast(), n_entries)
            })),
        }
    }
}

impl core::fmt::Debug for Elf<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: Add debugging info for struct
        f.debug_struct("Elf").finish()
    }
}
