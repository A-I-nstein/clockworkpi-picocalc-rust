# RP2350 SD Card and USB Serial Example - PicoCalc SPI SD Card Reader

This program demonstrates how to interface with an SD card using the SPI protocol on an RP2350 microcontroller. It utilizes the `rp235x-hal` for hardware abstraction, `embedded-sdmmc` for SD card communication, and `usbd-serial` to report information over a USB serial connection.

The code configures the SPI0 peripheral on GPIO pins 16 (MISO), 17 (CS), 18 (SCK), and 19 (MOSI). It also sets up a USB serial device to communicate with a host computer.

The main loop of the program periodically attempts to interact with the SD card. Every five seconds, it reports the card's size and lists the contents of the root directory of the first partition, sending this information over the USB serial port. After each attempt, it incrementally increases the SPI bus speed by 100 kHz to test the card's read performance at different frequencies.