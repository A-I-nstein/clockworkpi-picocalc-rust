# RP2350 USB Serial and PWM Control Example - PicoCalc PWM Speakers

This project demonstrates setting up a USB serial port and controlling Pulse Width Modulation (PWM) on the Raspberry Pi RP2350 microcontroller using Rust. It leverages the `rp235x-hal` crate for hardware abstraction, and `usb-device` and `usbd-serial` for USB communication.

The device enumerates as a virtual COM port on the host computer. Every second, it performs the following sequence:
- Sets up PWM channel A (connected to GPIO26) with a frequency of 100 Hz and a 50% duty cycle for 1 second. It sends `Success L` to the serial port on success, or an error message on failure. After a short delay, it turns off PWM channel A.
- Sets up PWM channel B (connected to GPIO27) with a frequency of 1000 Hz and a 50% duty cycle for 0.5 seconds. It sends `Success R` to the serial port on success, or an error message on failure. After a short delay, it turns off PWM channel B.