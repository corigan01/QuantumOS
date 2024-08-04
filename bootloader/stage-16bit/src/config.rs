#[repr(C)]
#[derive(Default, Debug)]
pub struct BootloaderConfig<'a> {
    bootloader32: &'a str,
    bootloader64: &'a str,
    kernel: &'a str,
    expected_vbe_mode: Option<(u32, u32)>,
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
