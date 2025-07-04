# RP2350 USB Serial and I2C Communication Example - PicoCalc I2C Keyboard

This project demonstrates setting up a USB serial port and I2C communication on the Raspberry Pi RP2350 microcontroller using Rust. It utilizes the `rp235x-hal` crate for hardware abstraction, and `usb-device` and `usbd-serial` for USB communication.

The device enumerates as a virtual COM port on the host computer. It attempts to communicate via I2C with a device at address `0x1f`, reading from register `0x09`. The retrieved value is then sent over the USB serial port every second. If the I2C communication fails, the error is reported via the serial port instead.