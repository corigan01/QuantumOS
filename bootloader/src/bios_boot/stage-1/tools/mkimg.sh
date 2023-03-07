#!/bin/bash

sudo sh rmimg.sh

dd if=/dev/zero of=disk.img bs=512 count=1048510

sed -e 's/\s*\([\+0-9a-zA-Z]*\).*/\1/' << EOF | sudo fdisk disk.img
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

sudo mkfs.fat -F 32 /dev/loop0p1
sudo mkfs.ext2 /dev/loop0p2

mkdir fs
mkdir fs/boot
mkdir fs/root

sudo mount /dev/loop0p1 fs/boot
#sudo mount /dev/loop0p2 fs/root

sudo touch myfile.txt fs/boot
sudo dd if=/dev/random of=fs/boot/myfile.txt count=10 bs=10


