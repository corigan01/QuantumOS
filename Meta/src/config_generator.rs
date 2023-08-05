/*
  ____                 __
 / __ \__ _____ ____  / /___ ____ _
/ /_/ / // / _ `/ _ \/ __/ // /  ' \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/
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

use std::fmt::{Display, Formatter};

pub struct BiosBootConfig {
    pub stage2_filepath: String,
    pub stage3_filepath: String,
    pub kernel_address: String,
    pub kernel_filepath: String,
    pub video_mode_preferred: (usize, usize),
}

impl BiosBootConfig {
    const KERNEL_FILE_LOCATION_KEY: &'static str = "KERNEL_ELF";
    const KERNEL_START_LOCATION_KEY: &'static str = "KERNEL_BEGIN";
    const NEXT_2_STAGE_LOCATION_KEY: &'static str = "NEXT_2_STAGE_BIN";
    const NEXT_3_STAGE_LOCATION_KEY: &'static str = "NEXT_3_STAGE_BIN";
    const VIDEO_MODE_KEY: &'static str = "VIDEO";
}

impl Display for BiosBootConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}={}\n{}={}\n{}={}\n{}={}\n{}={}x{}\n",
            Self::KERNEL_START_LOCATION_KEY,
            self.kernel_address,
            Self::KERNEL_FILE_LOCATION_KEY,
            self.kernel_filepath,
            Self::NEXT_2_STAGE_LOCATION_KEY,
            self.stage2_filepath,
            Self::NEXT_3_STAGE_LOCATION_KEY,
            self.stage3_filepath,
            Self::VIDEO_MODE_KEY,
            self.video_mode_preferred.0,
            self.video_mode_preferred.1
        )
    }
}

