use anyhow::{anyhow, Context, Error, Result};
use fatfs::{FileSystem, FsOptions};
use mbrman::{MBRPartitionEntry, MBR};
use std::fs::FileType;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use walkdir::WalkDir;

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
        self.root_img.write_all(&data).await?;

        Ok(())
    }

    pub async fn dir_to_fat(&mut self, dir_path: &Path) -> Result<()> {
        let fs_sectors = ((50 * 1024 * 1024) / 512) + 1;
        let fs_start = self
            .mbr
            .find_optimal_place(fs_sectors)
            .ok_or(anyhow!("Could not find optimal place for FAT-fs"))?;

        self.mbr[2] = MBRPartitionEntry {
            boot: mbrman::BOOT_INACTIVE,
            first_chs: mbrman::CHS::empty(),
            sys: 0x83,
            last_chs: mbrman::CHS::empty(),
            starting_lba: fs_start,
            sectors: fs_sectors,
        };

        let mut root_img = self
            .root_img
            .try_clone()
            .await
            .context("Failed to clone root_img")?
            .into_std()
            .await;
        let mut fat_slice = fscommon::StreamSlice::new(
            &mut root_img,
            fs_start as u64 * 512,
            (fs_start as u64 + fs_sectors as u64) * 512,
        )?;

        fatfs::format_volume(
            &mut fat_slice,
            fatfs::FormatVolumeOptions::new()
                .bytes_per_sector(512)
                .bytes_per_cluster(4096)
                .total_sectors(fs_sectors)
                .fats(2)
                .drive_num(0x80)
                .volume_label(*b"Q-BOOT     "),
        )?;

        let fat = fatfs::FileSystem::new(&mut fat_slice, FsOptions::new())?;
        let root_dir = fat.root_dir();

        for dir in WalkDir::new(dir_path).into_iter() {
            let dir = dir.context("Failed to walk dir for filesystem building")?;
            let fat_path = dir
                .path()
                .strip_prefix(dir_path)
                .context("Failed to create fat_path")?
                .to_str()
                .unwrap();

            if fat_path.is_empty() {
                continue;
            }

            if dir.file_type().is_dir() {
                root_dir
                    .create_dir(fat_path)
                    .context("Failed to create fat_path")?;
                continue;
            }

            let mut real_file = File::open(dir.path())
                .await
                .context("Cannot open real file")?;
            let mut file_data = Vec::new();
            real_file
                .read_to_end(&mut file_data)
                .await
                .context("Cannot read real file")?;

            let mut fat_file = root_dir
                .create_file(fat_path)
                .context("Cannot create fat file")?;
            fat_file
                .write_all(&mut file_data)
                .context("Failed to write real file data into fat file")?;
        }

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

pub async fn create_bootloader_dir(
    name: &str,
    artifacts: impl Iterator<Item = (&Path, &Path)>,
) -> Result<PathBuf> {
    let target_dir = tmp_find_target().join(name);
    tokio::fs::create_dir_all(&target_dir)
        .await
        .context("Failed to create bootloader dir")?;

    for object in artifacts {
        let bootloader_path = target_dir.join(object.1);

        tokio::fs::create_dir_all(&bootloader_path.parent().ok_or(anyhow!(
            "Cannot get the parent of file {:?}",
            bootloader_path.as_path()
        ))?)
        .await?;
        tokio::fs::copy(object.0, bootloader_path)
            .await
            .context("Failed to copy object to bootloader dir")?;
    }

    Ok(target_dir.into())
}
