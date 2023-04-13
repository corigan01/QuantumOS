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

use bootloader::error::BootloaderError;

pub struct BootloaderConfig<'a> {
    stage2_filepath: Option<&'a str>,
    kernel_address: Option<usize>,
    kernel_filepath: Option<&'a str>,
    video_mode_preferred: Option<(usize, usize)>,
}

impl<'a> BootloaderConfig<'a> {
    const KERNEL_FILE_LOCATION_KEY: &'static str = "KERNEL_ELF";
    const KERNEL_START_LOCATION_KEY: &'static str = "KERNEL_BEGIN";
    const NEXT_STAGE_LOCATION_KEY: &'static str = "NEXT_STAGE_BIN";
    const VIDEO_MODE_KEY: &'static str = "VIDEO";

    const DEFAULT_KERNEL_LOCATION: u64 = 16 * 1024 * 1024;

    pub fn from_str(string: &'a str) -> Result<Self, BootloaderError> {
        let mut config = BootloaderConfig {
            stage2_filepath: None,
            kernel_address: None,
            kernel_filepath: None,
            video_mode_preferred: None,
        };

        for line in string.split('\n') {
            let mut split_line = line.split('=');
            match (split_line.next(), split_line.next()) {
                (Some(Self::KERNEL_FILE_LOCATION_KEY), Some(location_key)) => {
                    config.kernel_filepath = Some(location_key.as_ref())
                }
                (Some(Self::KERNEL_START_LOCATION_KEY), Some(location_key)) => {
                    config.kernel_address = Some(location_key.trim().parse().unwrap_or(0))
                }
                (Some(Self::NEXT_STAGE_LOCATION_KEY), Some(location_key)) => {
                    config.stage2_filepath = Some(location_key.as_ref())
                }
                (Some(Self::VIDEO_MODE_KEY), Some(location_key)) => {
                    let mut video_mode_split = location_key.split('x');
                    let mut video_mode = (0usize, 0usize);

                    if let (Some(x), Some(y)) = (video_mode_split.next(), video_mode_split.next()) {
                        video_mode.0 = x.trim().parse().unwrap_or(0);
                        video_mode.1 = y.trim().parse().unwrap_or(0);
                    } else {
                        continue;
                    }

                    config.video_mode_preferred = Some(video_mode);
                }

                _ => {}
            }
        }

        Ok(config)
    }

    pub fn get_kernel_address(&self) -> u64 {
        match self.kernel_address {
            Some(value) => {
                let value = value as u64;

                if value > 1024 * 1024 {
                    value
                } else {
                    value * 1024 * 1024
                }
            }
            _ => Self::DEFAULT_KERNEL_LOCATION,
        }
    }

    pub fn get_kernel_file_path(&self) -> &str {
        self.kernel_filepath.unwrap_or("/kernel.elf")
    }

    pub fn get_stage2_file_path(&self) -> &str {
        self.stage2_filepath.unwrap_or("/bootloader/stage2.bin")
    }

    pub fn get_recommended_video_info(&self) -> (usize, usize) {
        self.video_mode_preferred.unwrap_or((640, 480))
    }
}

#[cfg(debug)]
use core::fmt::{Debug, Formatter};

#[cfg(debug)]
impl<'a> Debug for BootloaderConfig<'a> {
    fn fmt(&self, f: &mut Formatter) -> ::core::fmt::Result {
        writeln!(f, "BootloaderConfig {{")?;

        if let Some(kernel_file_path) = self.kernel_filepath {
            writeln!(f, "    Kernel Path     : {}", kernel_file_path)?;
        }
        if let Some(kernel_start) = self.kernel_address {
            writeln!(f, "    Kernel Address  : {}", kernel_start)?;
        }
        if let Some(stage_path) = self.stage2_filepath {
            writeln!(f, "    Stage2 Path     : {}", stage_path)?;
        }
        if let Some(video_mode) = self.video_mode_preferred {
            writeln!(f, "    Preferred video : {}x{}", video_mode.0, video_mode.1)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
