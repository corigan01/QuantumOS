/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

 VGA TEXT MODE DRIVER
 =======================

 This file handles the raw pointer to the memory that controls the framebuffer.

 The frame buffer can be in a few different formats and we need to support each
 different type so we can work on any boot type or hardware.
*/

pub mod text_mode;

pub enum FramebufferType {
    TextMode,
    PixelMode
}

