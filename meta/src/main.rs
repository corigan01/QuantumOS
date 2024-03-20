#![feature(async_closure)]
#![feature(async_fn_traits)]

use anyhow::{Context, Error, Result};
use clap::Parser;
use futures::executor::block_on;
use std::path::Path;

use crate::{artifacts::build_project, disk::DiskImgBaker};

mod artifacts;
mod cmdline;
mod disk;

async fn build() -> Result<()> {
    let artifacts = build_project().await.context("Failed to build artifacts")?;
    println!("{:#?}", artifacts);

    let mut disk = DiskImgBaker::new().await?;
    disk.write_bootsector(&artifacts.bootsector).await?;
    disk.write_stage16(&artifacts.stage_16).await?;

    disk.finish_and_write().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            build().await?;
            todo!("build")
        }
        cmdline::TaskOption::Run => {
            build().await?;
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }

    Ok(())
}
