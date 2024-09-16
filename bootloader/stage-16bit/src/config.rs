/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

#[repr(C)]
#[derive(Default, Debug)]
pub struct BootloaderConfig<'a> {
    pub bootloader32: &'a str,
    pub bootloader64: &'a str,
    pub kernel: &'a str,
    pub expected_vbe_mode: Option<(u16, u16)>,
}

impl<'a> BootloaderConfig<'a> {
    pub fn parse_file(file: &'a str) -> Option<Self> {
        let mut config = BootloaderConfig::default();

        for (first_option, second_option) in file
            .split('\n')
            .into_iter()
            .filter(|line| !line.is_empty() && line.is_ascii())
            .filter_map(|line| {
                let mut option_split = line.split('=');
                match (option_split.next(), option_split.next()) {
                    (Some(first_str), Some(second_str)) => Some((first_str, second_str)),
                    _ => None,
                }
            })
        {
            match first_option {
                "bootloader32" => config.bootloader32 = second_option,
                "bootloader64" => config.bootloader64 = second_option,
                "kernel" => config.kernel = second_option,
                "vbe-mode" => {
                    let mut info_split = second_option.split('x');
                    let (horz_str, vert_str) = (
                        info_split.next().unwrap_or(""),
                        info_split.next().unwrap_or(""),
                    );

                    match (horz_str.parse(), vert_str.parse()) {
                        (Ok(horz_number), Ok(vert_number)) => {
                            config.expected_vbe_mode = Some((horz_number, vert_number))
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        Some(config)
    }
}
