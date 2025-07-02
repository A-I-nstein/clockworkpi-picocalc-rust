# RP2350 USB Serial Example

This project provides a minimal example of how to set up a USB serial port on the Raspberry Pi RP2350 microcontroller using Rust. It leverages the `rp235x-hal` crate for hardware abstraction and `usb-device` and `usbd-serial` for USB communication.

The device enumerates as a virtual COM port on the host computer and continuously sends `"hello world"` messages.