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

use quantum::bios_boot::BiosBootConfig;
use std::fs;
use std::process::Command;

fn main() {
    println!("Welcome to the Quantum World");

    let command_args: Vec<String> = std::env::args().collect();
    let noqemu = command_args.contains(&String::from("noqemu"));
    let kvm = command_args.contains(&String::from("kvm"));

    if noqemu {
        println!("Build only mode!");
    }

    let mut build_status = bios_boot();

    if build_status.is_err() {
        println!("Failed to build --> {:?}", build_status.err());
        clean_dont_care();

        println!("Attempting to re-run build...");
        build_status = bios_boot();
    }

    if !noqemu {
        let user_extra_args: Vec<String> = if kvm {
            let mut vec = Vec::new();

            vec.push(String::from("-enable-kvm"));

            vec
        } else {
            Default::default()
        };

        let _qemu = Command::new("qemu-system-i386")
            .arg("-d")
            .arg("cpu_reset")
            .arg("--no-shutdown")
            .arg("-m")
            .arg("256M")
            .args(user_extra_args)
            .arg("-drive")
            .arg(format!("format=raw,file={}", build_status.unwrap()))
            .stdout(std::process::Stdio::piped())
            .status();
    } else {
        println!("NOT RUNNING QEMU!");
    }

    println!("Done :)");
}

fn clean_dont_care() {
    // Now clean up
    let _ = fs::remove_dir_all("target/bootloader_dir");
    let _ = fs::remove_file("target/i386-quantum_loader/release/stage-1");
    let _ = quantum::bios_disk::delete_disk_img("target/fat.img".into());
}

fn bios_boot() -> Result<String, Box<dyn std::error::Error>> {
    let target = quantum::get_build_directory().map_err(|err| {
        eprintln!("Unable to get the `target` for which to build into.");
        err
    })?;

    let bootloader_directory = quantum::bios_boot::make_bootloader_dir(&target).map_err(|err| {
        eprintln!("Unable to create bootloader directory! {err}");
        err
    })?;

    let inner_config_directory = format!("{}/bootloader", bootloader_directory);

    let stage_1_path = quantum::bios_boot::build_stage_1().unwrap();
    let stage_2_path = quantum::bios_boot::build_stage_2().unwrap();
    let kernel = quantum::build_kernel().unwrap();

    let bootloader_config = BiosBootConfig {
        stage2_filepath: "/bootloader/stage2.bin".to_string(),
        kernel_address: "16".to_string(),
        kernel_filepath: "/kernel.elf".to_string(),
        video_mode_preferred: (1280, 720),
    };

    fs::create_dir(&inner_config_directory)?;
    fs::copy(
        stage_2_path,
        format!("{}/stage2.bin", &inner_config_directory),
    )?;

    fs::copy(kernel, format!("{}/kernel.elf", &bootloader_directory))?;
    // LARGE TEST KERNEL
    /* fs::copy(
        "/bin/qemu-aarch64",
        format!("{}/kernel.elf", &bootloader_directory),
    )?;*/

    quantum::bios_boot::make_config_file(&inner_config_directory, bootloader_config)?;

    let fat_img = quantum::bios_disk::create_fat_img_from_directory(
        &target,
        format!("{}/bootloader_dir", &target),
    )?;

    let disk_img = quantum::bios_disk::make_mbr_disk(&target, &fat_img, &stage_1_path)?;

    fs::remove_file(&fat_img)?;

    Ok(disk_img)
}
