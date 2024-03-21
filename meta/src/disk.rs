use anyhow::{Context, Error, Result};
use mbrman::{MBRPartitionEntry, MBR};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const DISK_IMG_SIZE: usize = 1024 * 1024 * 512;

// FIXME: Get the target folder
fn tmp_find_target() -> PathBuf {
    PathBuf::from("./target/").canonicalize().unwrap()
}

pub struct DiskImgBaker {
    root_img: File,
    mbr: MBR,
}

impl DiskImgBaker {
    pub async fn new() -> Result<Self> {
        let root_img = create_diskimg("disk", DISK_IMG_SIZE).await?;

        // FIXME: This surely is not okay?
        let mbr = MBR::new_from(
            &mut root_img.try_clone().await?.into_std().await,
            512,
            [b'Q', b'-', b'O', b'S'],
        )?;

        Ok(DiskImgBaker { root_img, mbr })
    }

    pub async fn write_bootsector(&mut self, bootsector: &Path) -> Result<()> {
        let mut bootsector = File::open(bootsector).await?;

        let mut data = Vec::new();
        bootsector.read_to_end(&mut data).await?;

        self.mbr.header.bootstrap_code.copy_from_slice(&data[..440]);
        self.mbr.header.boot_signature = [0x55, 0xaa];

        Ok(())
    }

    pub async fn write_stage16(&mut self, stage16: &Path) -> Result<()> {
        let mut stage = File::open(stage16).await?;

        let mut data = Vec::new();
        stage.read_to_end(&mut data).await?;

        let stage_sectors = (data.len() / 512) + 1;
        let stage_start = self
            .mbr
            .find_optimal_place(stage_sectors as u32)
            .ok_or(Error::msg("Could not find optimal place for Stage16"))?;

        self.mbr[1] = MBRPartitionEntry {
            boot: mbrman::BOOT_ACTIVE,
            first_chs: mbrman::CHS::empty(),
            sys: 0x83,
            last_chs: mbrman::CHS::empty(),
            starting_lba: stage_start,
            sectors: stage_sectors as u32,
        };

        self.root_img
            .seek(io::SeekFrom::Start(
                stage_start as u64 * self.mbr.sector_size as u64,
            ))
            .await?;
        self.root_img.write_all(&mut data).await?;

        Ok(())
    }

    pub async fn finish_and_write(mut self) -> Result<PathBuf> {
        let mut disk_img = self.root_img.into_std().await;
        self.mbr.write_into(&mut disk_img)?;
        disk_img.flush()?;

        Ok(tmp_find_target().join("img").join("disk.img"))
    }
}

async fn create_diskimg(name: &str, size: usize) -> Result<File> {
    let target_dir = tmp_find_target().join("img");
    tokio::fs::create_dir_all(&target_dir)
        .await
        .context("Failed to create directories")?;

    let file_name = target_dir.join(format!("{}.img", name));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_name)
        .await
        .context("failed to diskimg open file")?;
    file.set_len(size as u64)
        .await
        .context("Failed to set disk img file len")?;

    Ok(file)
}

pub async fn _write_file_into_fat(
    fat: &mut fatfs::FileSystem<std::fs::File>,
    src_path: &Path,
    dest_path: &str,
) -> Result<()> {
    let mut src_file = File::open(src_path).await?;

    // FIXME: If the file is too big, we should read it in chunks
    //        since this could be very memory intesive. However,
    //        the OS project is so small at the moment that this
    //        should not cause an issue.
    let mut data = Vec::new();
    src_file.read_to_end(&mut data).await?;

    let root = fat.root_dir();
    let _ = root.remove(dest_path);

    let mut dest_file = root.create_file(dest_path)?;
    dest_file.write_all(&data)?;

    dest_file.flush()?;

    Ok(())
}
