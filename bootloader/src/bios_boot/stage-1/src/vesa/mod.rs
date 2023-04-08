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

use crate::error::BootloaderError;
use crate::{bios_print, bios_println, convert_segmented_ptr};
use core::marker::PhantomData;

pub mod low_level_structs;

pub struct VesaMode {
    pub mode_id: usize,
    pub mode_data: low_level_structs::VesaModeInfo,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Res {
    pub x: usize,
    pub y: usize,
    pub depth: usize,
}

impl VesaMode {
    pub fn get_res(&self) -> Res {
        Res {
            x: self.mode_data.width as usize,
            y: self.mode_data.height as usize,
            depth: self.mode_data.bpp as usize,
        }
    }
}

pub struct BiosVesa<State = UnQuarried> {
    current_mode: Option<VesaMode>,
    info: low_level_structs::VesaDriverInfo,
    reserved: PhantomData<State>,
}

pub struct UnQuarried;
pub struct Quarried;

impl BiosVesa {
    pub fn new() -> BiosVesa<UnQuarried> {
        BiosVesa {
            current_mode: None,
            info: low_level_structs::VesaDriverInfo::new(),
            reserved: Default::default(),
        }
    }
}

impl BiosVesa<UnQuarried> {
    pub fn quarry(self) -> Result<BiosVesa<Quarried>, BootloaderError> {
        Ok(BiosVesa {
            current_mode: self.current_mode,
            info: low_level_structs::VesaDriverInfo::quarry()?,
            reserved: Default::default(),
        })
    }
}

impl BiosVesa<Quarried> {
    pub fn run_on_all_supported_modes<Function>(
        &self,
        mut runnable: Function,
    ) -> Result<VesaMode, BootloaderError>
    where
        Function: FnMut(&VesaMode) -> bool,
    {
        let supported_modes = self.info.get_all_supported_modes()?;
        bios_print!("COCK ");

        for mode in supported_modes {
            let mode_info_packed = low_level_structs::VesaModeInfo::quarry(mode);

            if let Ok(mode_info) = mode_info_packed {
                let safe_vesa_mode = VesaMode {
                    mode_id: mode,
                    mode_data: mode_info,
                };

                if runnable(&safe_vesa_mode) {
                    return Ok(safe_vesa_mode);
                }
            }
        }

        Err(BootloaderError::NoValid)
    }

    fn get_mode_with_id(&self, raw_mode_id: usize) -> Result<VesaMode, BootloaderError> {
        self.run_on_all_supported_modes(|mode| mode.mode_id == raw_mode_id)
    }

    pub fn find_closest_mode(&self, resolution: Res) -> Result<VesaMode, BootloaderError> {
        let mut x_offset = usize::MAX;
        let mut y_offset = usize::MAX;
        let mut depth_offset = usize::MAX;

        let mut closest_mode_id = 0;

        self.run_on_all_supported_modes(|mode| {
            let res = mode.get_res();

            let x_dff = res.x.abs_diff(resolution.x);
            let y_diff = res.y.abs_diff(resolution.y);
            let depth_diff = res.depth.abs_diff(resolution.depth);

            if x_dff < x_offset && y_diff < y_diff && depth_diff < depth_offset {
                x_offset = x_dff;
                y_offset = y_diff;
                depth_offset = depth_diff;

                closest_mode_id = mode.mode_id;
            }

            false
        })?;

        self.get_mode_with_id(closest_mode_id)
    }
}
