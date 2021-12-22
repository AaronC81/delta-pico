// Modified from:
// https://github.com/hathach/tinyusb/blob/master/examples/device/msc_dual_lun

// This code doesn't match the other peripherals very closely - it's C rather than C++ after all.
// That's because TinyUSB has a lot of callbacks which need to be implemented as C functions.

#include "tusb.h"
#include "bsp/board.h"

#include <stdint.h>
#include <string.h>

#include "usb_mass_storage.h"

// Keep track of whether mass storage has been ejected
bool usb_mass_storage_ejected = false;

// Device descriptor
tusb_desc_device_t const usb_mass_storage_device =
{
    .bLength            = sizeof(tusb_desc_device_t),
    .bDescriptorType    = TUSB_DESC_DEVICE,
    .bcdUSB             = 0x0200,
    .bDeviceClass       = 0x00,
    .bDeviceSubClass    = 0x00,
    .bDeviceProtocol    = 0x00,
    .bMaxPacketSize0    = CFG_TUD_ENDPOINT0_SIZE,

    .idVendor           = 0xCafe,
    .idProduct          = 0xdafe,
    .bcdDevice          = 0x0100,

    .iManufacturer      = 0x01,
    .iProduct           = 0x02,
    .iSerialNumber      = 0x03,

    .bNumConfigurations = 0x01
};

// Getter function for device descriptor
uint8_t const * tud_descriptor_device_cb(void)
{
    return (uint8_t const *) &usb_mass_storage_device;
}

// Getter for string descriptors - copy them into a UTF-16 buffer so they live long enough
static uint16_t usb_mass_storage_string_buffer[32];
uint16_t const* tud_descriptor_string_cb(uint8_t index, uint16_t langid)
{
    const char *strings[] = {
        // Vendor             Product       Serial
        "Aaron Christiansen", "Delta Pico", "123456789012"
    };

    char *string;

    switch (index) {
    // Language ID
    case 0:
        usb_mass_storage_string_buffer[1] = 0x09;
        usb_mass_storage_string_buffer[2] = 0x04;
        return usb_mass_storage_string_buffer;
    
    // Product, vendor, or serial
    case 1: case 2: case 3:
        string = strings[index - 1];
        break;

    // No idea!
    default:
        return NULL;
    }

    // 2 byte header - type of descriptor (string), and length including header
    uint8_t length = strlen(string);
    usb_mass_storage_string_buffer[0] = (TUSB_DESC_STRING << 8) | (2 * length + 2);
    
    // Write string - can't use `strcpy` because we're copying 8-bit chars into 16-bit buffer slots
    for (uint8_t i = 0; i < length; i++) {
        usb_mass_storage_string_buffer[i + 1] = string[i];
    }

    return usb_mass_storage_string_buffer;
}


// Some stuff related to config that I don't particularly understand, so haven't tweaked too much
enum
{
    ITF_NUM_MSC,
    ITF_NUM_TOTAL
};

#define CONFIG_TOTAL_LEN    (TUD_CONFIG_DESC_LEN + TUD_MSC_DESC_LEN)

#define EPNUM_MSC_OUT   0x01
#define EPNUM_MSC_IN    0x81

uint8_t const desc_fs_configuration[] =
{
    // Config number, interface count, string index, total length, attribute, power in mA
    TUD_CONFIG_DESCRIPTOR(1, ITF_NUM_TOTAL, 0, CONFIG_TOTAL_LEN, 0x00, 100),

    // Interface number, string index, EP Out & EP In address, EP size
    TUD_MSC_DESCRIPTOR(ITF_NUM_MSC, 0, EPNUM_MSC_OUT, EPNUM_MSC_IN, 64),
};

// Invoked when received GET CONFIGURATION DESCRIPTOR
// Application return pointer to descriptor
// Descriptor contents must exist long enough for transfer to complete
uint8_t const * tud_descriptor_configuration_cb(uint8_t index)
{
    return desc_fs_configuration;
}

// This device only has one LUN - i.e. one drive shows up when we connect it
uint8_t tud_msc_get_maxlun_cb(void) { return 1; }

// We're ready as long as we haven't been ejected
bool tud_msc_test_unit_ready_cb(uint8_t lun) { return !usb_mass_storage_ejected; }

// LUN is writeable
bool tud_msc_is_writable_cb (uint8_t lun) { return true; }

// Callback when host asks for info about our LUN
void tud_msc_inquiry_cb(uint8_t lun, uint8_t vendor_id[8], uint8_t product_id[16], uint8_t product_rev[4]) {
    const char vid[] = "Delta Pico";
    const char pid[] = "Mass Storage";
    const char rev[] = "1.0";

    memcpy(vendor_id  , vid, strlen(vid));
    memcpy(product_id , pid, strlen(pid));
    memcpy(product_rev, rev, strlen(rev));
}

// Callback when host asks for capacity of our LUN
void tud_msc_capacity_cb(uint8_t lun, uint32_t* block_count, uint16_t* block_size) {
    *block_count = USB_MASS_STORAGE_BLOCK_NUM;
    *block_size  = USB_MASS_STORAGE_BLOCK_SIZE;
}

// Invoked when received Start Stop Unit command
// - Start = 0 : stopped power mode, if load_eject = 1 : unload disk storage
// - Start = 1 : active mode, if load_eject = 1 : load disk storage
bool tud_msc_start_stop_cb(uint8_t lun, uint8_t power_condition, bool start, bool load_eject) {
    if (load_eject)
    {
        if (!start) {
            usb_mass_storage_ejected = true;
        } else {
            return !usb_mass_storage_ejected;
        }
    }

    return true;
}

// Callback when host wants to read data
int32_t tud_msc_read10_cb(uint8_t lun, uint32_t lba, uint32_t offset, void* buffer, uint32_t bufsize) {
    // Unsure why *reads* need to check capacity, but the TinyUSB example put this here, so I won't
    // touch it!
    if (lba >= USB_MASS_STORAGE_BLOCK_NUM) return -1;
    
    // Copy data into library-provided buffer
    uint8_t const* addr = usb_mass_storage_fat12_filesystem + (lba * USB_MASS_STORAGE_BLOCK_SIZE) + offset;
    memcpy(buffer, addr, bufsize);

    return bufsize;
}


// Callback when host wants to write data
int32_t tud_msc_write10_cb(uint8_t lun, uint32_t lba, uint32_t offset, uint8_t* buffer, uint32_t bufsize)
{
    // Error if we're run out of capacity (host tries to write to a block we don't have)
    if (lba >= USB_MASS_STORAGE_BLOCK_NUM) return -1;

    // Copy buffer into our filesystem
    uint8_t* addr = usb_mass_storage_fat12_filesystem + (lba * USB_MASS_STORAGE_BLOCK_SIZE) + offset;
    memcpy(addr, buffer, bufsize);

    return bufsize;
}

// Callback when host wants to do something not handled by another callback
int32_t tud_msc_scsi_cb(uint8_t lun, uint8_t const scsi_cmd[16], void* buffer, uint16_t bufsize)
{
    // The return value of this function is really a "response length", but none of the things we
    // respond to actually need a message, so it's effectively a binary code of 0 = success, -1 =
    // error
    int status = 0;

    switch (scsi_cmd[0])
    {
    case SCSI_CMD_PREVENT_ALLOW_MEDIUM_REMOVAL:
        // Host is about to read/write - don't really need to do anything with that information
        status = 0;
        break;

    default:
        // Dunno! Error
        tud_msc_set_sense(lun, SCSI_SENSE_ILLEGAL_REQUEST, 0x20, 0x00);
        status = -1;
        break;
    }

    return status;
}
