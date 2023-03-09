#!/bin/bash

sudo umount /dev/loop0 /dev/loop0p2 /dev/loop0p1 /dev/loop0*
sudo losetup -d /dev/loop0

sudo rm disk.img
sudo rm -r fs