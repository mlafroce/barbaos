#!/usr/bin/env bash

LOOP_DEVICE=$(sudo losetup --partscan --find --show hdd.img)
if [ $? -ne 0 ]; then
    echo "Error: Failed to set up loop device"
    exit 1
fi

echo "Loop device set up as: $LOOP_DEVICE"

PARTITION="${LOOP_DEVICE}p1"

if [ ! -e "$PARTITION" ]; then
    echo "Error: Partition $PARTITION not found. Did you forgot to format the disk?"
    sudo losetup -d "$LOOP_DEVICE"  # Clean up loop device
    exit 1
fi

MOUNT_POINT="hdd"
mkdir -p "$MOUNT_POINT"
sudo mount "$PARTITION" "$MOUNT_POINT"
