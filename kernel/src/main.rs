/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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
#![allow(dead_code)]

use core::panic::PanicInfo;
use qk_alloc::heap::alloc::KernelHeap;
use qk_alloc::heap::set_global_alloc;
use qk_alloc::usable_region::UsableRegion;

use quantum_lib::{debug_print, debug_println, kernel_entry, rect};
use quantum_lib::address_utils::PAGE_SIZE;
use quantum_lib::address_utils::physical_address::PhyAddress;
use quantum_lib::address_utils::region::{MemoryRegion, MemoryRegionType};
use quantum_lib::boot::boot_info::KernelBootInformation;
use quantum_lib::bytes::Bytes;
use quantum_lib::com::serial::{SerialBaud, SerialDevice, SerialPort};
use quantum_lib::debug::add_connection_to_global_stream;
use quantum_lib::debug::stream_connection::StreamConnectionBuilder;
use quantum_lib::gfx::{Pixel, PixelLocation, rectangle::Rect};
use quantum_lib::possibly_uninit::PossiblyUninit;

use quantum_os::clock::rtc::update_and_get_time;

use owo_colors::OwoColorize;
use qk_alloc::string::String;

static mut SERIAL_CONNECTION: PossiblyUninit<SerialDevice> = PossiblyUninit::new_lazy(|| {
    SerialDevice::new(SerialPort::Com1, SerialBaud::Baud115200).unwrap()
});


kernel_entry!(main);

fn setup_serial_debug() {
    let serial = unsafe { &mut SERIAL_CONNECTION };

    let connection = StreamConnectionBuilder::new()
        .console_connection()
        .add_connection_name("Serial COM1")
        .does_support_scrolling(true)
        .add_outlet(serial.get_ref().unwrap())
        .build();

    add_connection_to_global_stream(connection).unwrap();

    debug_println!("Welcome to Quantum OS! {}\n", update_and_get_time());
}

fn main(boot_info: &KernelBootInformation) {
    setup_serial_debug();

    let mut physical_memory_map = boot_info.get_physical_memory().clone();
    let mut virtual_memory_map = boot_info.get_virtual_memory().clone();

    physical_memory_map.consolidate().unwrap();
    virtual_memory_map.consolidate().unwrap();

    debug_println!("Virtual Memory Map:\n{virtual_memory_map:?}");
    debug_println!("Physical Memory Map:\n{physical_memory_map:?}");

    let total_phy: u64 = physical_memory_map.total_mem_for_type(MemoryRegionType::Usable).into();
    let total_pages: u64 = total_phy / (PAGE_SIZE as u64);

    debug_println!("Total Usable Physical Memory {} ({} -- 4k Pages)",
        Bytes::from(total_phy),
        total_pages
    );

    // FIXME: The tmp alloc should be dynamic
    let init_alloc_begin = 2 * Bytes::MIB;
    let init_alloc_size = Bytes::from(1 * Bytes::MIB);

    debug_print!("\nCreating Init Heap Allocator at (ptr: 0x{:x} size: {}) ... ",
        init_alloc_begin, init_alloc_size
    );

    let is_within = physical_memory_map.is_within(MemoryRegion::from_distance(
        PhyAddress::try_from(init_alloc_begin).unwrap(),
        init_alloc_size,
        MemoryRegionType::Usable
    ));

    assert!(is_within, "Failed, the region is not within the memory map! TODO: Make the Init region dynamic!");

    let usable_region = unsafe {
        UsableRegion::from_raw_parts(
            init_alloc_begin as *mut u8,
            init_alloc_size.into()
        )
    }.unwrap();

    let new_kernel_heap = KernelHeap::new(usable_region)
        .expect("Unable to create init kernel allocator");

    set_global_alloc(new_kernel_heap);

    debug_println!("{}", "OK".bright_green().bold());

    let string_test = String::from("OK".bright_green().bold());
    debug_println!("Test String {}", string_test.as_str());

    debug_println!("\nUsing Bootloader framebuffer");
    let mut framebuffer = boot_info.framebuffer.clone();

    let clear_display_color = Pixel::from_hex(0x111111);

    debug_print!("Clearing Display with {:?} ... ", clear_display_color);
    framebuffer.fill_entire(clear_display_color);
    debug_println!("{}", "OK".bright_green().bold());

    debug_print!("Drawing Boot Graphics ... ");
    framebuffer.draw_built_in_text(PixelLocation::new(0, 0), Pixel::WHITE, "QuantumOS");
    framebuffer.draw_rect(rect!(0, 15 ; 150, 2), Pixel::WHITE);
    debug_println!("{}", "OK".bright_green().bold());

    debug_println!("\n\nDone!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("{}", info);
    loop {}
}