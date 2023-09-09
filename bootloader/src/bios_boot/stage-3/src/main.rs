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

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr;
use bootloader::boot_info::BootInfo;

use quantum_lib::debug::{add_connection_to_global_stream, set_panic};
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::{debug_print, debug_println};
use quantum_lib::address_utils::physical_address::PhyAddress;
use quantum_lib::address_utils::region::{MemoryRegion, MemoryRegionType};
use quantum_lib::address_utils::region_map::RegionMap;
use quantum_lib::address_utils::virtual_address::VirtAddress;
use quantum_lib::boot::boot_info::KernelBootInformation;
use quantum_lib::com::serial::{SerialBaud, SerialDevice, SerialPort};
use quantum_lib::elf::{ElfHeader, ElfArch, ElfBits, ElfSegmentType};
use quantum_lib::x86_64::PrivlLevel;
use quantum_lib::x86_64::registers::{Segment, SegmentRegs};
use quantum_utils::human_bytes::HumanBytes;
use quantum_lib::gfx::frame_info::FrameInfo;
use quantum_lib::gfx::FramebufferPixelLayout;
use quantum_lib::gfx::linear_framebuffer::LinearFramebuffer;
use quantum_lib::possibly_uninit::PossiblyUninit;

use stage_3::debug::{clear_framebuffer, display_string, setup_framebuffer};

static mut SERIAL_CONNECTION: PossiblyUninit<SerialDevice> = PossiblyUninit::new_lazy(|| {
    SerialDevice::new(SerialPort::Com1, SerialBaud::Baud115200).unwrap()
});

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(boot_info: u64) -> ! {
    let boot_info_ref = BootInfo::from_ptr(boot_info as usize);

    unsafe { SegmentRegs::set_data_segments(Segment::new(2, PrivlLevel::Ring0)); }

    let video_info = boot_info_ref.get_video_information();

    let framebuffer = video_info.framebuffer;
    let x_res = video_info.x;
    let y_res = video_info.y;
    let bbp = video_info.depth;

    setup_framebuffer(
        framebuffer,
        x_res as usize,
        y_res as usize,
        bbp as usize,
        true,
    );

    clear_framebuffer();

    let stream_connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_simple_outlet(display_string)
        .add_connection_name("VGA DEBUG")
        .does_support_scrolling(true)
        .build();

    let serial = unsafe { &mut SERIAL_CONNECTION };

    let connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_connection_name("Serial")
        .add_who_using("Stage3")
        .does_support_scrolling(true)
        .add_outlet(serial.get_mut_ref().unwrap())
        .build();

    add_connection_to_global_stream(stream_connection).unwrap();
    add_connection_to_global_stream(connection).unwrap();

    debug_println!("Quantum Bootloader! (Stage3) [64 bit]");

    main(boot_info_ref);
    panic!("Stage3 should not return!");
}

fn parse_kernel_elf(kernel_elf: &ElfHeader) -> Option<(u32, MemoryRegion<PhyAddress>)> {
    let kernel_arch = kernel_elf.elf_arch()?;
    let kernel_bits = kernel_elf.elf_bits()?;
    let mut lower_kernel_end = u64::MAX;
    let mut higher_kernel_end = 0;

    debug_print!("Kernel arch={:?}, bits={:?} ...", kernel_arch, kernel_bits);

    if !matches!(kernel_arch, ElfArch::X86_64) || !matches!(kernel_bits, ElfBits::Bit64) {
        debug_println!("Err");
        return None;
    }

    debug_println!("OK");

    let header_amount = kernel_elf.elf_number_of_entries_in_program_table()?;
    let entry_point = kernel_elf.elf_entry_point()? as u32;

    debug_println!(
        "Kernel Info: p-header=(O: {} S: {} B: {}) s-header=(O: {}, S: {}, B: {}) e-point={:x}",
        kernel_elf.elf_program_header_table_position()?,
        header_amount,
        kernel_elf.elf_size_of_entry_in_program_table()?,
        kernel_elf.elf_section_header_table_position()?,
        kernel_elf.elf_number_of_entries_in_section_table()?,
        kernel_elf.elf_size_of_entry_in_section_table()?,
        entry_point
    );


    // TODO: This is a super simple ELF loader and should be improved
    for i in 0..header_amount {
        let header_idx = i as usize;
        let header = kernel_elf.get_program_header(header_idx)?;

        let expected_address = header.data_expected_address();
        let real_memory_size = header.size_in_mem();
        let end_of_expected_address = expected_address + real_memory_size;

        debug_print!(
            "Header {} = '{:10x?}' -- {:?} => F: {} M: {} O: {} Vaddr: {:x}",
            header_idx,
            header.type_of_segment(),
            header.flags(),
            header.size_in_elf(),
            real_memory_size,
            header.data_offset(),
            expected_address
        );

        let header_type = header.type_of_segment();
        let kernel_raw_data = kernel_elf.get_raw_data_slice();

        if matches!(header_type, ElfSegmentType::Load) {
            lower_kernel_end =
                if lower_kernel_end > expected_address
                { expected_address } else { lower_kernel_end };

            higher_kernel_end =
                if higher_kernel_end < end_of_expected_address
                { end_of_expected_address } else { higher_kernel_end };

            let loader_ptr = expected_address as *mut u8;

            let data_size = header.size_in_elf() as usize;
            let ac_data_offset = header.data_offset() as usize;

            let data_slice = &kernel_raw_data[ac_data_offset..(data_size + ac_data_offset)];

            for (i, byte) in data_slice.iter().enumerate() {
                unsafe {
                    *loader_ptr.add(i) = *byte;
                }
            }

            debug_println!("      \tLOADED");

        } else {
            debug_println!("       \tSKIP");
        }
    }

    let region = MemoryRegion::new(
        PhyAddress::new(lower_kernel_end).unwrap(),
        PhyAddress::new(higher_kernel_end).unwrap(),
        MemoryRegionType::KernelCode
    );

    Some((entry_point, region))
}

static mut PHS_REGION_MAP: PossiblyUninit<RegionMap<PhyAddress>> = PossiblyUninit::new_lazy(|| RegionMap::new());
static mut VRT_REGION_MAP: PossiblyUninit<RegionMap<VirtAddress>> = PossiblyUninit::new_lazy(|| RegionMap::new());
static mut KRNL_BOOT_INFO: PossiblyUninit<KernelBootInformation> = PossiblyUninit::new();

fn main(boot_info: &BootInfo) {
    debug_println!("Starting to parse kernel ELF...");

    let kernel_info = boot_info.get_kernel_entry();
    let kernel_slice = unsafe { core::slice::from_raw_parts_mut(kernel_info.ptr as *mut u8, kernel_info.size as usize) };

    let kernel_elf = ElfHeader::from_bytes(kernel_slice).unwrap();

    let (entry_point, kernel_region) = parse_kernel_elf(&kernel_elf).unwrap();

    // TODO: Yeah lets maybe not have the stack just hard coded here :)
    debug_println!("Zero-ing Kernel Stack region");
    let stack_ptr = 15 * HumanBytes::MIB;
    for ptr in (stack_ptr - (2 * HumanBytes::MIB))..stack_ptr {
        unsafe { ptr::write(ptr as *mut u8, 0_u8); }
    }

    debug_print!("Rebuilding Regions... ");
    let region_map = unsafe { PHS_REGION_MAP.get_mut_ref().unwrap() };
    for e820_entry in unsafe { boot_info.get_memory_map() } {

        let region_type = match e820_entry.entry_type {
            1 => MemoryRegionType::Usable,
            2 => MemoryRegionType::Reserved,
            3 | 4 => MemoryRegionType::Uefi,
            5 => MemoryRegionType::UnavailableMemory,
            _ => MemoryRegionType::Unknown
        };

        let start_address = PhyAddress::new(e820_entry.address).unwrap();
        let size_bytes = HumanBytes::from(e820_entry.len);

        let region= MemoryRegion::from_distance(start_address, size_bytes, region_type);

        region_map.add_new_region(region).unwrap();
    }
    region_map.add_new_region(kernel_region).unwrap();

    let stack_region = MemoryRegion::new(
        PhyAddress::new(stack_ptr - HumanBytes::MIB).unwrap(),
        PhyAddress::new(stack_ptr).unwrap(),
        MemoryRegionType::KernelStack
    );

    region_map.add_new_region(stack_region).unwrap();

    // TODO: Get this region map from stage-2
    let virtual_region_map = unsafe { VRT_REGION_MAP.get_mut_ref().unwrap() };
    let virtual_region = MemoryRegion::<VirtAddress>::new(
        VirtAddress::new(0).unwrap(),
        VirtAddress::new(5 * HumanBytes::GIB).unwrap(),
        MemoryRegionType::Unknown
    );

    virtual_region_map.add_new_region(virtual_region).unwrap();

    debug_println!("Regions Built!");

    let old_framebuffer = boot_info.get_video_information();
    let stride = old_framebuffer.x * old_framebuffer.depth;
    let total_bytes = stride * old_framebuffer.y;

    // TODO: Get the pixel layout from stage-1
    let frame_info = FrameInfo::new(
        old_framebuffer.x as usize,
        old_framebuffer.y as usize,
        old_framebuffer.depth as usize,
        stride as usize,
        total_bytes as usize,
        FramebufferPixelLayout::BGR
    );

    let video = LinearFramebuffer::new(old_framebuffer.framebuffer as *mut u8, frame_info);

    unsafe {
        KRNL_BOOT_INFO.set(KernelBootInformation::new(
            region_map.clone(),
            virtual_region_map.clone(),
            video
        ));
    }

    let kernel_info = unsafe { KRNL_BOOT_INFO.get_ref().unwrap().send_as_u64() };


    debug_println!("Kernel Entry Point 0x{:x}", entry_point);

    debug_println!("Calling Kernel!");
    clear_framebuffer();

    debug_println!();
    debug_println!("The Kernel has not setup the framebuffer");
    debug_println!("========================================");
    debug_println!("Quantum-Bootloader has setup this framebuffer for you!");
    debug_println!("Please use the included boot info structure to gather");
    debug_println!("the needed information to draw to this framebuffer.");
    debug_println!("Find more info here: https://github.com/corigan01/QuantumOS");

    debug_println!("\nKernel [entry: 0x{:x}, stack: 0x{:x}, size: {}]", entry_point, stack_ptr, kernel_region.bytes());
    debug_println!("Cpu: x86_64 64-bit | Kern: {:?} {:?}", kernel_elf.elf_arch().unwrap(), kernel_elf.elf_bits().unwrap());

    debug_println!("\nCalling '_start' in elf at 0x{:x}", entry_point);

    unsafe {
        asm!(
            "mov rsp, {stack}",
            "jmp {kern:r}",
            in("rdi") kernel_info,
            kern = in(reg) entry_point,
            stack = in(reg) stack_ptr
        );
    }
}

#[panic_handler]
#[cold]
#[allow(dead_code)]
fn panic(info: &PanicInfo) -> ! {
    set_panic();
    debug_println!("");

    debug_println!("\nStage-3 PANIC ============\n{}\n\n", info);
    loop {}
}