#!/bin/bash

sudo sh rmimg.sh

dd if=/dev/zero of=disk.img bs=512 count=1048510

chmod 777 disk.img

sed -e 's/\s*\([\+0-9a-zA-Z]*\).*/\1/' << EOF | fdisk disk.img
  o     # clear the in memory partition table
  n     # new partition
  p     # primary partition
  1     # partition number 1
        # default - start at beginning of disk
  +50M  # 50 MB boot parttion
  n     # new partition
  p     # primary partition
  2     # partion number 2
        # default, start immediately after preceding partition
        # default, extend partition to end of disk
  a     # make a partition bootable
  1     # bootable partition is partition 1
  p     # print the in-memory partition table
  w     # write the partition table
  q     # and we're done
EOF

sudo losetup -P --show -v /dev/loop0 disk.img
sudo chmod 777 /dev/loop0*

sudo mkdosfs /dev/loop0p1
sudo mkfs.ext2 /dev/loop0p2

mkdir fs
mkdir fs/boot
mkdir fs/root
mkdir "fs/boot/DISK NOT MOUNTED"
mkdir "fs/root/DISK NOT MOUNTED"

sudo mount /dev/loop0p1 fs/boot
sudo mount -text2 /dev/loop0p2 fs/root

sudo mkdir fs/boot/bootloader
echo "# BOOTLOADER CONFIG FILE" | sudo tee -a fs/boot/bootloader/bootloader.cfg
echo "KERNEL_ELF=/kernel.elf" | sudo tee -a fs/boot/bootloader/bootloader.cfg
echo "NEXT_STAGE_BIN=/bootloader/stage2.bin" | sudo tee -a fs/boot/bootloader/bootloader.cfg
echo "VIDEO=1280x720" | sudo tee -a fs/boot/bootloader/bootloader.cfg
echo "KERNEL_BEGIN=16" | sudo tee -a fs/boot/bootloader/bootloader.cfg
echo "" | sudo tee -a fs/boot/bootloader/bootloader.cfg

# Unmount to sync changes to disk and make sure that we dont have to wait for the OS to cache changes
sudo umount /dev/loop0p1 /dev/loop0p2
sudo mount /dev/loop0p1 fs/boot
sudo mount -text2 /dev/loop0p2 fs/root