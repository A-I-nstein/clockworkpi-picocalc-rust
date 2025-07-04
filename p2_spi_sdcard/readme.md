# Work In Progress

# RP2350 SD Card and USB Serial Example - PicoCalc SPI SD Card Reader

This project provides a minimal example of how to set up an SD card interface and a USB serial port on the Raspberry Pi RP2350 microcontroller using Rust. It leverages the `rp235x-hal` crate for hardware abstraction, `embedded-sdmmc` for SD card interaction, and `usb-device` and `usbd-serial` for USB communication.

The device enumerates as a virtual COM port on the host computer and periodically attempts to read the size of the inserted SD card, reporting the result (or any errors) over the serial connection. It also tries to open the first volume on the SD card.