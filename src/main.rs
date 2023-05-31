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
use owo_colors::OwoColorize;

fn main() {
    println!("     {} Welcome to the Quantum World", "Quantum".green().bold());

    let cargo = std::env::var("CARGO").unwrap_or("cargo".into());
    let current_dir = std::env::current_dir().unwrap();
    let target = format!("{}/target", current_dir.display());

    let command_args: Vec<String> = std::env::args().collect();
    let noqemu = command_args.contains(&String::from("noqemu"));
    let kvm = command_args.contains(&String::from("kvm"));
    let debug_int = command_args.contains(&String::from("debug-int"));
    let debug = command_args.contains(&String::from("debug"));
    let test_libs = command_args.contains(&String::from("test-libs"));

    if test_libs {
        println!("     {} Testing all Libs!", "Quantum".green().bold());

        let _lowlevel_lib = Command::new(cargo.clone())
            .current_dir("lib/lowlevel_lib")
            .arg("test")
            .arg(format!("--target-dir={}/lowlevel_lib", target))
            .stdout(std::process::Stdio::inherit())
            .status()
            .unwrap();

        let _stacked_lib = Command::new(cargo.clone())
            .current_dir("lib/stacked")
            .arg("test")
            .arg(format!("--target-dir={}/stacked", target))
            .stdout(std::process::Stdio::inherit())
            .status()
            .unwrap();

    } else {
        if noqemu {
            println!("     {} Build only mode!", "Quantum".green().bold());
        }

        let mut build_status = bios_boot(false);

        if build_status.is_err() {
            println!("Failed to build --> {:?}", build_status.err());
            clean_dont_care();

            println!("Attempting to re-run build...");
            build_status = bios_boot(false);
        }

        if !noqemu {
            let mut user_extra_args: Vec<String> = Default::default();

            if kvm {
                user_extra_args.push(String::from("-enable-kvm"));
            }

            if debug_int {
                user_extra_args.push(String::from("-d"));
                user_extra_args.push(String::from("int"));
            }

            if debug {
                user_extra_args.push(String::from("-s"));
                user_extra_args.push(String::from("-S"));
            }

            println!("     {} Starting QEMU", "Quantum".green().bold());

            let _qemu = Command::new("qemu-system-x86_64")
                .args(user_extra_args)
                .arg("-d")
                .arg("cpu_reset")
                .arg("--no-shutdown")
                .arg("-m")
                .arg("256M")
                .arg("-display")
                .arg("gtk")
                .arg("-k")
                .arg("en-us")
                .arg("-serial")
                .arg("stdio")
                .arg("-nic")
                .arg("none")
                .arg("-drive")
                .arg(format!("format=raw,file={}", build_status.unwrap()))
                .stdout(std::process::Stdio::inherit())
                .status();
        } else {
            println!("     {} NOT RUNNING QEMU!", "Quantum".yellow().bold());
        }
    }

    println!("\n{} All Jobs Complete!", "Quantum".green().bold());
}

fn clean_dont_care() {
    // Now clean up
    let _ = fs::remove_dir_all("target/bootloader_dir");
    let _ = fs::remove_file("target/i386-quantum_loader/release/stage-1");
    let _ = fs::remove_file("target/i686-quantum_loader/release/stage-2");
    let _ = fs::remove_file("target/x86_64-quantum_loader/release/stage-3");

    let _ = quantum::bios_disk::delete_disk_img("target/fat.img".into());
}

fn bios_boot(kernel_in_test_mode: bool) -> Result<String, Box<dyn std::error::Error>> {
    let target = quantum::get_build_directory().map_err(|err| {
        eprintln!("Unable to get the `target` for which to build into.");
        err
    })?;

    let _ = fs::remove_dir_all("target/bootloader_dir");

    let bootloader_directory = quantum::bios_boot::make_bootloader_dir(&target).map_err(|err| {
        eprintln!("Unable to create bootloader directory! {err}");
        err
    })?;

    let inner_config_directory = format!("{}/bootloader", bootloader_directory);

    let stage_1_path = quantum::bios_boot::build_stage_1().unwrap();
    let stage_2_path = quantum::bios_boot::build_stage_2().unwrap();
    let stage_3_path = quantum::bios_boot::build_stage_3().unwrap();

    let kernel = quantum::build_kernel(kernel_in_test_mode).unwrap();



    let bootloader_config = BiosBootConfig {
        stage2_filepath: "/bootloader/stage2.bin".to_string(),
        stage3_filepath: "/bootloader/stage3.bin".to_string(),
        kernel_address: "16".to_string(),
        kernel_filepath: "/kernel.elf".to_string(),
        video_mode_preferred: (1280, 720),
    };

    fs::create_dir(&inner_config_directory)?;
    fs::copy(
        stage_2_path,
        format!("{}/stage2.bin", &inner_config_directory),
    )?;
    fs::copy(
        stage_3_path,
        format!("{}/stage3.bin", &inner_config_directory),
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
