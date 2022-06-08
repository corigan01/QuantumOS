/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

 VGA TEXT MODE DRIVER
 =======================

 This file is here to contain the vga text mode driver functions such as drawing to the screen.

*/

use volatile::Volatile;

pub mod low_level;

pub enum FramebufferType {
    TextMode,
    PixelMode
}

pub enum OutputType {
    TextOnly,
    PixelOnly
}

pub fn init_kernel_vga(framebuffer_type: FramebufferType, output_type: OutputType,
                        buffer: u64, frame_size: (u32, u32)) {

    match framebuffer_type {
        TextMode => {

        },

        PixelMode => {

        },

        _ => {

        }
    }

}