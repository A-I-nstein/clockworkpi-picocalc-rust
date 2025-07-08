# clockworkpi-picocalc-rust

**Embedded Rust development for the ClockworkPi PicoCalc commercial product.** This repository houses drivers and applications specifically designed for the PicoCalc.

## Getting Started

### How to run the programs?
Using the `runner` configuration in the project's `.cargo/config.toml` file, you can directly build and flash your Rust applications to the Pico 2W using standard `cargo` commands.

1.  Install / prepare the prerequisites.
2.  Connect the USB port of the Pico 2W to your computer.
3.  Navigate into the required project directory (e.g., `cd p1_usb_serial`).
4.  Execute the `cargo run` command.

### Building and Flashing with `cargo run`
The `.cargo/config.toml` file in this project is configured to use `picotool` as the `runner` for the `thumbv8m.main-none-eabihf` target. This means `cargo run` will automatically:
1.  Build your Rust application.
2.  Invoke `picotool` to flash the compiled file to the Pico 2W.

    - **To run in development profile** (for faster compilation and debugging): `cargo run`
    - **To run in release profile** (for optimized code and smaller size, recommended for deployment): `cargo run --release`

**Important Note on BOOTSEL mode:**
For `picotool load` to succeed, the Pico 2W generally needs to be in **BOOTSEL mode** (hold `BOOTSEL` button while plugging in the USB cable). `cargo run` will execute the `picotool` command, and `picotool` will attempt to find a device in this mode. If you encounter issues, always try manually putting the Pico into BOOTSEL mode first before running `cargo run`.


## Prerequisites

### Hardware
- The ClockworkPi PicoCalc - [clockworkpi.com](https://www.clockworkpi.com/picocalc), [Github](https://github.com/clockworkpi/PicoCalc)
- The Raspberry Pi Pico 2 W - [Hardware Guide](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html#pico2w-technical-specification)

### Software Installations
- Install "The Rust Programming Language" - [Installation Guide](https://rust-lang.github.io/rustup/installation/index.html)
- Pico-SDK - [Download Link](https://github.com/raspberrypi/pico-sdk/releases)
- Picotool - [Download Link](https://github.com/raspberrypi/picotool/releases)
- PuTTY - [Download Link](https://www.putty.org/)

### rustup Setup
- Add the necessary target for ARM embedded development: `rustup target add thumbv8m.main-none-eabihf`

## Troubleshooting
- **`picotool: command not found`**: Ensure `picotool` is installed, built, and its executable is in your system's PATH.
- **`No accessible RP2350 devices in BOOTSEL mode were found.`**: Make sure you are holding the `BOOTSEL` button while plugging in the Pico 2W to explicitly enter bootloader mode.
- **Compilation fails with "no such target"**: Double-check that you have added the `thumbv8m.main-none-eabihf` target using `rustup`.
- **Unexpected behavior after flashing**: Try manually putting the Pico into BOOTSEL mode and flashing. If issues persist, a `picotool erase` before loading can sometimes resolve stubborn flash corruption issues (be aware this erases the entire flash memory).

## Components Explored
- [X] USB Serial
- [ ] SPI SD Card
- [X] SPI Display
- [X] I2C Keyboard
- [X] PWM Speakers
- [ ] QSPI PSRAM

## License
This project is licensed under the MIT License.