/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfArch {
    NoSpecific,
    Sparc,
    X86,
    Mips,
    PowerPC,
    Arm,
    SuperH,
    Ia64,
    X86_64,
    Aarch64,
    RiscV
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfBits {
    Bit32,
    Bit64
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfEndian {
    LittleEndian,
    BigEndian
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfType {
    Relocatable,
    Executable,
    Shared,
    Core
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfErr {
    InvalidLen,
    InvalidArch,
    NotAnElf,
    InvalidAlignment
}


pub struct ElfHeader<'a> {
    raw_data: &'a [u8]
}

impl<'a> ElfHeader<'a> {
    pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ElfErr> {
        // Min Header len
        if bytes.len() < 54 {
            return Err(ElfErr::InvalidLen);
        }

        let header = ElfHeader {raw_data: bytes};

        // Check if header is elf
        match header.is_elf() {
            true => {
                // Make sure we can get the identify the bits in the elf,
                if let Some(bits) = header.elf_bits() {
                    // ensure the len is enough for 64 bit
                    if matches!(bits, ElfBits::Bit64) && bytes.len() < 64 {
                        Err(ElfErr::InvalidLen)
                    } else {
                        // Only if it meets all checks can it pass
                        Ok(header)
                    }
                } else {
                    Err(ElfErr::InvalidArch)
                }
            },
            false => Err(ElfErr::NotAnElf)
        }
    }

    fn u16_from_data(&self, pos: usize) -> u16 {
        let mut u16_bytes = [0_u8; 2];
        u16_bytes.copy_from_slice(&self.raw_data[pos..(pos + 2)]);

        u16::from_le_bytes(u16_bytes)
    }

    fn u32_from_data(&self, pos: usize) -> u32 {
        let mut u32_bytes = [0_u8; 4];
        u32_bytes.copy_from_slice(&self.raw_data[pos..(pos + 4)]);

        u32::from_le_bytes(u32_bytes)
    }

    fn u64_from_data(&self, pos: usize) -> u64 {
        let mut u64_bytes = [0_u8; 8];
        u64_bytes.copy_from_slice(&self.raw_data[pos..(pos + 8)]);

        u64::from_le_bytes(u64_bytes)
    }

    fn both_arch_return_from_pos(&self, pos_if_u32: usize, pos_if_u64: usize) -> Option<u64> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u32_from_data(pos_if_u32) as u64)
            }
            ElfBits::Bit64 => {
                Some(self.u64_from_data(pos_if_u64))
            }
        }
    }

    pub fn is_elf(&self) -> bool {
        for (index, magic_bytes) in self.raw_data[..4].iter().enumerate() {
            let check_byte = Self::ELF_MAGIC[index];

            if check_byte != *magic_bytes {
                return false;
            }
        }

        true
    }

    pub fn elf_bits(&self) -> Option<ElfBits> {
        let bits_data = self.raw_data[4];

        match bits_data {
            1 => Some(ElfBits::Bit32),
            2 => Some(ElfBits::Bit64),

            _ => None
        }
    }

    pub fn elf_endian(&self) -> Option<ElfEndian> {
        let bits_data = self.raw_data[5];

        match bits_data {
            1 => Some(ElfEndian::LittleEndian),
            2 => Some(ElfEndian::BigEndian),

            _ => None
        }
    }

    pub fn elf_header_version(&self) -> usize {
        self.raw_data[6] as usize
    }

    pub fn elf_os_abi(&self) -> usize {
        self.raw_data[7] as usize
    }

    pub fn elf_type(&self) -> Option<ElfType> {
        let bits_data = self.u16_from_data(16);

        match bits_data {
            1 => Some(ElfType::Relocatable),
            2 => Some(ElfType::Executable),
            3 => Some(ElfType::Shared),
            4 => Some(ElfType::Core),

            _ => None
        }
    }

    pub fn elf_arch(&self) -> Option<ElfArch> {
        let bits_data = self.u16_from_data(18);

        match bits_data {
            0 => Some(ElfArch::NoSpecific),
            2 => Some(ElfArch::Sparc),
            3 => Some(ElfArch::X86),
            8 => Some(ElfArch::Mips),
            0x14 => Some(ElfArch::PowerPC),
            0x28 => Some(ElfArch::Arm),
            0x2A => Some(ElfArch::SuperH),
            0x32 => Some(ElfArch::Ia64),
            0x3E => Some(ElfArch::X86_64),
            0xB7 => Some(ElfArch::Aarch64),
            0xF3 => Some(ElfArch::RiscV),

            _ => None
        }
    }

    pub fn elf_version(&self) -> usize {
        self.u32_from_data(20) as usize
    }

    pub fn elf_entry_point(&self) -> Option<u64> {
        self.both_arch_return_from_pos(24, 24)
    }

    pub fn elf_program_header_table_position(&self) -> Option<u64> {
        self.both_arch_return_from_pos(28, 32)
    }

    pub fn elf_section_header_table_position(&self) -> Option<u64> {
        self.both_arch_return_from_pos(32, 40)
    }

    pub fn elf_flags(&self) -> Option<u32> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u32_from_data(36))
            }
            ElfBits::Bit64 => {
                Some(self.u32_from_data(48))
            }
        }
    }

    pub fn elf_header_size(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(40))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(52))
            }
        }
    }

    pub fn elf_size_of_entry_in_program_table(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(42))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(54))
            }
        }
    }

    pub fn elf_number_of_entries_in_program_table(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(44))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(56))
            }
        }
    }

    pub fn elf_size_of_entry_in_section_table(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(46))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(58))
            }
        }
    }

    pub fn elf_number_of_entries_in_section_table(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(48))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(60))
            }
        }
    }

    pub fn elf_start_of_section_names_offset(&self) -> Option<u16> {
        match self.elf_bits()? {
            ElfBits::Bit32 => {
                Some(self.u16_from_data(50))
            }
            ElfBits::Bit64 => {
                Some(self.u16_from_data(62))
            }
        }
    }

    pub fn get_program_header(&self, idx: usize) -> Option<ProgramHeader> {
        if idx > self.elf_number_of_entries_in_program_table()? as usize {
            return None;
        }

        let bits = self.elf_bits()?;
        let header_in_elf_offset= self.elf_program_header_table_position()? as usize;
        let bytes_per_table = self.elf_size_of_entry_in_program_table()? as usize;
        let offset_of_headers = bytes_per_table * idx;

        let total_offset_bytes = header_in_elf_offset + offset_of_headers;

        if total_offset_bytes > self.raw_data.len() {
            return None;
        }

        let slice = &self.raw_data[total_offset_bytes..(total_offset_bytes + bytes_per_table)];

        if let Ok(header) = ProgramHeader::new(slice, bits) {
            Some(header)
        } else {
            None
        }
    }

    pub fn get_raw_data_slice(&self) -> &[u8] {
        self.raw_data
    }
}

/*
Flags: 1 = executable, 2 = writable, 4 = readable.

*/
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfHeaderFlags {
    Executable,
    Writable,
    Readable,
    None
}


/*
32 Bit version:
Position	Value
0-3	    Type of segment (see below)
4-7	    The offset in the file that the data for this segment can be found (p_offset)
8-11	Where you should start to put this segment in virtual memory (p_vaddr)
12-15	Undefined for the System V ABI
16-19	Size of the segment in the file (p_filesz)
20-23	Size of the segment in memory (p_memsz)
24-27	Flags (see below)
28-31	The required alignment for this section (must be a power of 2)

64 bit version:
Position	Value
0-3	    Type of segment (see below)
4-7	    Flags (see below)
8-15	The offset in the file that the data for this segment can be found (p_offset)
16-23	Where you should start to put this segment in virtual memory (p_vaddr)
24-31	Undefined for the System V ABI
32-39	Size of the segment in the file (p_filesz)
40-47	Size of the segment in memory (p_memsz)
48-55	The required alignment for this section (must be a power of 2)
*/

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ElfSegmentType {
    IgnoreEntry,
    Load,
    Dynamic,
    RequiresInterp,
    Note,
    Phdr,
    Unknown(usize)
}

pub struct ProgramHeader<'a> {
    raw_data: &'a [u8],
    bits: ElfBits
}

impl<'a> ProgramHeader<'a> {
    pub fn new(bytes: &'a [u8], bits: ElfBits) -> Result<Self, ElfErr> {
        let bytes_len = bytes.len();

        if matches!(bits, ElfBits::Bit64) && bytes_len < 56 {
            return Err(ElfErr::InvalidLen);
        }
        if matches!(bits, ElfBits::Bit32) && bytes_len < 32 {
            return Err(ElfErr::InvalidLen);
        }

        Ok(Self {
            raw_data: bytes,
            bits: bits,
        })
    }

    fn u32_from_data(&self, pos: usize) -> u32 {
        let mut u32_bytes = [0_u8; 4];
        u32_bytes.copy_from_slice(&self.raw_data[pos..(pos + 4)]);

        u32::from_le_bytes(u32_bytes)
    }

    fn u64_from_data(&self, pos: usize) -> u64 {
        let mut u64_bytes = [0_u8; 8];
        u64_bytes.copy_from_slice(&self.raw_data[pos..(pos + 8)]);

        u64::from_le_bytes(u64_bytes)
    }

    fn both_arch_sized_return_from_pos(&self, pos_if_u32: usize, pos_if_u64: usize) -> u64 {
        match self.bits {
            ElfBits::Bit32 => {
                self.u32_from_data(pos_if_u32) as u64
            }
            ElfBits::Bit64 => {
                self.u64_from_data(pos_if_u64)
            }
        }
    }

    fn both_u32_sized_return_from_pos(&self, pos_if_u32: usize, pos_if_u64: usize) -> u32 {
        match self.bits {
            ElfBits::Bit32 => {
                self.u32_from_data(pos_if_u32)
            }
            ElfBits::Bit64 => {
                self.u32_from_data(pos_if_u64)
            }
        }
    }

    pub fn type_of_segment(&self) -> ElfSegmentType {
        let seg_type_number = self.u32_from_data(0);

        match seg_type_number {
            0 => ElfSegmentType::IgnoreEntry,
            1 => ElfSegmentType::Load,
            2 => ElfSegmentType::Dynamic,
            3 => ElfSegmentType::RequiresInterp,
            4 => ElfSegmentType::Note,
            6 => ElfSegmentType::Phdr,

            _ => ElfSegmentType::Unknown(seg_type_number as usize)
        }
    }

    #[allow(unused_assignments)]
    pub fn flags(&self) -> [ElfHeaderFlags; 3] {
        let flags_number = self.both_u32_sized_return_from_pos(24, 4);
        let mut flag_builder = [ElfHeaderFlags::None; 3];

        let mut flags_found = 0;

        if flags_number & 0b001 > 0 {
            flag_builder[flags_found] = ElfHeaderFlags::Executable;
            flags_found += 1;
        }
        if flags_number & 0b010 > 0 {
            flag_builder[flags_found] = ElfHeaderFlags::Writable;
            flags_found += 1;
        }
        if flags_number & 0b100 > 0 {
            flag_builder[flags_found] = ElfHeaderFlags::Readable;
            flags_found += 1;
        }

        flag_builder
    }

    // p_offset
    pub fn data_offset(&self) -> u64 {
        self.both_arch_sized_return_from_pos(4, 8)
    }

    // p_vadder
    pub fn data_expected_address(&self) -> u64 {
        self.both_arch_sized_return_from_pos(8, 16)
    }

    // p_filesz
    pub fn size_in_elf(&self) -> u64 {
        self.both_arch_sized_return_from_pos(16, 32)
    }

    // p_memsz
    pub fn size_in_mem(&self) -> u64 {
        self.both_arch_sized_return_from_pos(20, 40)
    }

    pub fn section_alignment(&self) -> u64 {
        self.both_arch_sized_return_from_pos(28, 48)
    }
}
