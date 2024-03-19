use clap::Parser;
use futures::executor::block_on;
use std::path::Path;

use crate::artifacts::build_project;

mod artifacts;
mod cmdline;
mod disk;

fn main() {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            todo!("build")
        }
        cmdline::TaskOption::Run => {
            let artifacts = block_on(build_project()).expect("Failed to build Quantum Project");
            println!("{:#?}", artifacts);
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }
}
