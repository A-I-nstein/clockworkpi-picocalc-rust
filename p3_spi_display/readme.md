# Work In Progress

# RP2350 ILI9486Rgb666 Display Example - PicoCalc SPI Display

This project provides a minimal example of how to initialize and control an ILI9486Rgb666 LCD display with an RP2350 microcontroller using Rust. It leverages the `rp235x-hal` crate for hardware abstraction and `mipidsi` for display interaction.

The code configures the necessary SPI communication on GPIO pins 10 (SCK), 11 (MOSI), and 12 (MISO), along with GPIO pins 14 (Data/Command), 13 (Chip Select), and 15 (Reset) for display control. After initializing the display, it clears the entire screen and fills it with a solid green color.