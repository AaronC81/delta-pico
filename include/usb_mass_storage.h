#pragma once

#define USB_MASS_STORAGE_BLOCK_NUM 16
#define USB_MASS_STORAGE_BLOCK_SIZE 512

extern bool usb_mass_storage_ejected;
uint8_t (*usb_mass_storage_fat12_filesystem)[USB_MASS_STORAGE_BLOCK_NUM][USB_MASS_STORAGE_BLOCK_SIZE];