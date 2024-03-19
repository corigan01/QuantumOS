use artifacts::build_bootloader;
use clap::Parser;
use futures::executor::block_on;
use std::path::Path;

mod artifacts;
mod cmdline;

fn main() {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            todo!("build")
        }
        cmdline::TaskOption::Run => {
            let artifacts = block_on(build_bootloader(Path::new("./"), false)).unwrap();
            println!("{:#?}", artifacts);
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }
}
