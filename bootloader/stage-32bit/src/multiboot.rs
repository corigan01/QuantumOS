/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

Copyright 2025 Gavin Kellam

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

use bootloader::{Stage16toStage32, MAX_MEMORY_MAP_ENTRIES};
use core::{mem::ManuallyDrop, ptr::null};
use lldebug::logln;

// TODO: impl multiboot2 structures
#[allow(dead_code)]
mod multiboot2_headers {
    const MULTIBOOT2_HEADER_MAGIC: u32 = 0xe85250d6;
    const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36d76289;

    #[repr(C)]
    pub(crate) struct Multiboot2Header {
        pub(crate) magic: u32,
        pub(crate) flags: u32,
        pub(crate) header_len: u32,
        pub(crate) checksum: u32,
    }

    pub(crate) mod header_tags {
        #[repr(C)]
        pub(crate) struct HeaderTag {
            pub(crate) kind: u16,
            pub(crate) flags: u16,
            pub(crate) size: u32,
        }

        #[repr(C)]
        pub(crate) struct InfoRequest {
            pub(crate) tag: HeaderTag,
            pub(crate) requests: u32,
        }

        #[repr(C)]
        pub(crate) struct Address {
            pub(crate) tag: HeaderTag,
            pub(crate) header_addr: u32,
            pub(crate) load_addr: u32,
            pub(crate) load_end_addr: u32,
            pub(crate) bss_end_addr: u32,
        }

        #[repr(C)]
        pub(crate) struct EntryAddress {
            pub(crate) tag: HeaderTag,
            pub(crate) entry_addr: u32,
        }

        #[repr(C)]
        pub(crate) struct ConsoleFlags {
            pub(crate) tag: HeaderTag,
            pub(crate) console_flags: u32,
        }

        #[repr(C)]
        pub(crate) struct Framebuffer {
            pub(crate) tag: HeaderTag,
            pub(crate) width: u32,
            pub(crate) height: u32,
            pub(crate) depth: u32,
        }

        #[repr(C)]
        pub(crate) struct ModuleAlign {
            pub(crate) tag: HeaderTag,
        }

        #[repr(C)]
        pub(crate) struct Relocatable {
            pub(crate) tag: HeaderTag,
            pub(crate) min_addr: u32,
            pub(crate) max_addr: u32,
            pub(crate) align: u32,
            pub(crate) preference: u32,
        }
    }

    #[repr(C)]
    pub(crate) struct Color {
        red: u8,
        green: u8,
        blue: u8,
    }

    #[repr(C)]
    pub(crate) struct MemoryEntry {
        addr: u64,
        len: u64,
        kind: u32,
        zero: u32,
    }

    #[repr(C)]
    pub(crate) struct VbeInfoBlock {
        external: [u8; 512],
    }

    #[repr(C)]
    pub(crate) struct VbeModeInfoBlock {
        external: [u8; 256],
    }

    pub(crate) mod tags {
        use core::mem::ManuallyDrop;

        #[repr(C)]
        pub(crate) struct Tag {
            kind: u32,
            size: u32,
        }

        #[repr(C)]
        pub(crate) struct TagString {
            tag: Tag,
            str: [core::ffi::c_char; 0],
        }

        #[repr(C)]
        pub(crate) struct Module {
            tag: Tag,
            mod_start: u32,
            mod_end: u32,
            cmd_line: [core::ffi::c_char; 0],
        }

        #[repr(C)]
        pub(crate) struct BasicMemInfo {
            tag: Tag,
            mem_lower: u32,
            mem_upper: u32,
        }

        #[repr(C)]
        pub(crate) struct BootDev {
            tag: Tag,
            biosdev: u32,
            slice: u32,
            part: u32,
        }

        #[repr(C)]
        pub(crate) struct Mmap {
            tag: Tag,
            entry_size: u32,
            entry_version: u32,
            entries: [super::MemoryEntry; 0],
        }

        #[repr(C)]
        pub(crate) struct Vbe {
            tag: Tag,
            vbe_mode: u16,
            vbe_interface_segment: u16,
            vbe_interface_offset: u16,
            vbe_interface_length: u16,
            control_info: super::VbeInfoBlock,
            mode_info: super::VbeModeInfoBlock,
        }

        #[repr(C)]
        pub(crate) struct FramebufferCommon {
            tag: Tag,
            address: u32,
            pitch: u32,
            width: u32,
            height: u32,
            bpp: u8,
            framebuffer_kind: u8,
            reserved: u16,
        }

        #[repr(C)]
        pub(crate) struct Framebuffer {
            common: FramebufferCommon,
            opt: FramebufferCfgOption,
        }

        #[repr(C)]
        pub(crate) union FramebufferCfgOption {
            palette_info: ManuallyDrop<FramebufferPaletteInfo>,
            color_info: ManuallyDrop<FramebufferColorInfo>,
        }

        #[repr(C)]
        pub(crate) struct FramebufferPaletteInfo {
            number_of_colors: u16,
            color_palette: [super::Color; 0],
        }

        #[repr(C)]
        pub(crate) struct FramebufferColorInfo {
            red_pos: u8,
            red_mask_size: u8,
            green_pos: u8,
            green_mask_size: u8,
            blue_pos: u8,
            blue_mask_size: u8,
        }

        #[repr(C)]
        pub(crate) struct ElfSections {
            tag: Tag,
            num: u32,
            entry_size: u32,
            shndx: u32,
            sections: [core::ffi::c_char; 0],
        }

        #[repr(C)]
        pub(crate) struct Apm {
            tag: Tag,
            version: u16,
            code_segment: u16,
            offset: u32,
            code_segment_16: u16,
            data_segment: u16,
            flags: u16,
            code_segment_len: u16,
            code_segment_16_len: u16,
            data_segment_len: u16,
        }

        #[repr(C)]
        pub(crate) struct Efi32 {
            tag: Tag,
            pointer: u32,
        }

        #[repr(C)]
        pub(crate) struct Efi64 {
            tag: Tag,
            pointer: u64,
        }

        #[repr(C)]
        pub(crate) struct SmBios {
            tag: Tag,
            major: u8,
            minor: u8,
            reserved: [u8; 6],
            tables: [u8; 0],
        }

        #[repr(C)]
        pub(crate) struct OldAcpi {
            tag: Tag,
            rsdp: [u8; 0],
        }

        #[repr(C)]
        pub(crate) struct NewAcpi {
            tag: Tag,
            rsdp: [u8; 0],
        }

        #[repr(C)]
        pub(crate) struct Network {
            tag: Tag,
            dhcpack: [u8; 0],
        }

        #[repr(C)]
        pub(crate) struct EfiMap {
            tag: Tag,
            descriptor_size: u32,
            descriptor_vers: u32,
            efi_mmap: [u8; 0],
        }

        #[repr(C)]
        pub(crate) struct Efi32ih {
            tag: Tag,
            pointer: u32,
        }

        #[repr(C)]
        pub(crate) struct Efi64ih {
            tag: Tag,
            pointer: u64,
        }

        #[repr(C)]
        pub(crate) struct LoadBaseAddr {
            tag: Tag,
            load_base_addr: u32,
        }
    }

    #[repr(u32)]
    pub enum IsaKind {
        I386 = 0,
        Mips32 = 4,
    }

    #[repr(u8)]
    pub enum HeaderTagKind {
        End = 0,
        InformationRequest = 1,
        Address = 2,
        EntryAddress = 3,
        ConsoleFlags = 4,
        Framebuffer = 5,
        ModuleAlign = 6,
        EfiBs = 7,
        EntryAddressEfi32 = 8,
        EntryAddressEfi64 = 9,
        Relocateable = 10,
    }

    #[repr(u8)]
    pub enum TagKind {
        End = 0,
        Cmdline = 1,
        BootLoaderName = 2,
        Module = 3,
        BasicMemInfo = 4,
        BootDev = 5,
        Mmap = 6,
        Vbe = 7,
        Framebuffer = 8,
        ElfSections = 9,
        Apm = 10,
        Efi32 = 11,
        Efi64 = 12,
        Smbios = 13,
        AcpiOld = 14,
        AcpiNew = 15,
        Network = 16,
        EfiMmap = 17,
        EfiBs = 18,
        Efi32ih = 19,
        Efi64ih = 20,
        LoadBaseAddr = 21,
    }

    pub struct HeaderFlags(u16);

    impl HeaderFlags {
        pub const OPTIONAL_FLAG: HeaderFlags = HeaderFlags(1);
    }

    trait HeaderTag: Sized {
        const HEADER_TAG_KIND: HeaderTagKind;
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot_header")]
pub static MULTIBOOT_HEADER: SimpleMultiboot1Header = SimpleMultiboot1Header::new();

#[unsafe(no_mangle)]
#[unsafe(link_section = ".stack")]
pub static INIT_STACK: [u8; 16384] = [0; 16384];

#[repr(C, align(8))]
pub struct SimpleMultiboot1Header {
    magic: u32,
    flags: u32,
    checksum: u32,
    reserved: [u32; 8],
    vid_mode: u32,
}

impl SimpleMultiboot1Header {
    const MULTIBOOT1_MAGIC: u32 = 0x1BADB002;
    const MULTIBOOT1_FLAGS_ALIGN: u32 = 1;
    const MULTIBOOT1_FLAGS_MEM_INFO: u32 = 2;
    const MULTIBOOT1_FLAGS_GFX_MODE: u32 = 4;

    pub const fn new() -> Self {
        let flags = Self::MULTIBOOT1_FLAGS_ALIGN
            | Self::MULTIBOOT1_FLAGS_MEM_INFO
            | Self::MULTIBOOT1_FLAGS_GFX_MODE;
        Self {
            magic: Self::MULTIBOOT1_MAGIC,
            flags,
            checksum: (Self::MULTIBOOT1_MAGIC + flags).overflowing_neg().0,
            reserved: [0; 8],
            vid_mode: 32,
        }
    }
}

#[macro_export]
macro_rules! init_multiboot {
    () => {{
        unsafe {
            let multiboot_ptr: u32;
            core::arch::asm!(
                "mov esp, {stack}",
                out("ebx") multiboot_ptr,
                stack = in(reg) multiboot::INIT_STACK.as_ptr() as u32 + multiboot::INIT_STACK.len() as u32
            );

            debug_macro::debug_macro_init();
            multiboot::get_stage_to_stage_from_multiboot_header(multiboot_ptr as *const multiboot::Multiboot1Info)
        }
    }};
}

#[repr(C)]
pub struct Multiboot1Info {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    modules_count: u32,
    modules_address: u32,
    elf_sections_or_symbol: [u8; 16],
    mmap_length: u32,
    mmap_address: u32,
    drives_length: u32,
    drives_address: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_segment: u16,
    vbe_interface_offset: u16,
    vbe_ineterface_length: u16,
    framebuffer_addr: u32,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_bpp: u8,
    framebuffer_kind: u8,
    framebuffer_options: FramebufferCfgOption,
}

#[repr(C)]
union FramebufferCfgOption {
    palette_info: ManuallyDrop<FramebufferPaletteInfo>,
    color_info: ManuallyDrop<FramebufferColorInfo>,
}

#[repr(C)]
struct FramebufferPaletteInfo {
    palette_addr: u32,
    number_of_colors: u16,
}

#[repr(C)]
struct FramebufferColorInfo {
    red_pos: u8,
    red_mask_size: u8,
    green_pos: u8,
    green_mask_size: u8,
    blue_pos: u8,
    blue_mask_size: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct MmapEntry {
    size: u32,
    base_addr_low: u32,
    base_addr_hi: u32,
    length_low: u32,
    length_hi: u32,
    kind: u32,
}

pub fn get_stage_to_stage_from_multiboot_header(header: *const Multiboot1Info) -> Stage16toStage32 {
    if header == null() {
        panic!("Multiboot header was null!");
    }

    // This is to prevent the compiler from removing the boot magic and the stack
    let header_ptr = &raw const MULTIBOOT_HEADER.magic;
    let stack_ptr = &raw const INIT_STACK as *const u32;
    assert_eq!(
        unsafe { core::ptr::read_volatile(header_ptr) },
        SimpleMultiboot1Header::MULTIBOOT1_MAGIC
    );
    assert_eq!(unsafe { core::ptr::read_volatile(stack_ptr) }, 0);

    let header_ref = unsafe { &*header };
    logln!("Multiboot mode! Launched from bootloader {:?}!", unsafe {
        core::ffi::CStr::from_ptr(header_ref.boot_loader_name as *const i8)
    });

    let multiboot_mmap = unsafe {
        core::slice::from_raw_parts(
            header_ref.mmap_address as *const MmapEntry,
            header_ref.mmap_length as usize / size_of::<MmapEntry>(),
        )
    };

    // FIXME: We should make a more generic memory entry to pass around instead of
    //        using e820 mappings.
    let mut e820_map: [bios::memory::MemoryEntry; MAX_MEMORY_MAP_ENTRIES] =
        [unsafe { core::mem::zeroed() }; MAX_MEMORY_MAP_ENTRIES];
    e820_map
        .iter_mut()
        .zip(multiboot_mmap.iter())
        .for_each(|(e820, entry)| {
            e820.base_address = entry.base_addr_low as u64 | (entry.base_addr_hi as u64) << 32;
            e820.region_length = entry.length_low as u64 | (entry.length_hi as u64) << 32;
            e820.region_type = entry.kind;
        });

    // Qemu writes all of the PTRs and LENs of each of our bootloader compoenents into memory addr +1Mib
    //
    // You can find more details of this in the meta/main.rs file.
    let &[stage32_ptr, stage32_len, stage64_ptr, stage64_len, kernel_ptr, kernel_len, initfs_ptr, initfs_len] =
        // FIXME: I am not sure if this is the best way of passing these arguments in, but
        //        its also only for emulator booting so I think its fine for now. Maybe
        //        replace in the future?
        (unsafe { core::slice::from_raw_parts(0x100000 as *const u64, 8) })
    else {
        unreachable!("Cannot match compile time length amount of elements!");
    };

    Stage16toStage32 {
        bootloader_stack_ptr: (stack_ptr as u64, INIT_STACK.len() as u64),
        stage32_ptr: (stage32_ptr, stage32_len),
        stage64_ptr: (stage64_ptr, stage64_len),
        kernel_ptr: (kernel_ptr, kernel_len),
        initfs_ptr: (initfs_ptr, initfs_len),
        memory_map: e820_map,
        video_mode: None,
    }
}
