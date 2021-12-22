#pragma once

#define DISK_BLOCK_NUM 16
#define DISK_BLOCK_SIZE 512

extern bool usb_mass_storage_ejected;
uint8_t (*usb_mass_storage_fat12_filesystem)[DISK_BLOCK_NUM][DISK_BLOCK_SIZE];