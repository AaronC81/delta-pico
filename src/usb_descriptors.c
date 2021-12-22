// Modified from:
// https://github.com/hathach/tinyusb/blob/master/examples/device/msc_dual_lun/src/usb_descriptors.c

#include "tusb.h"

#include <stdint.h>
#include <string.h>

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
