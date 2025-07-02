#![no_std]
#![no_main]

use panic_halt as _;
use rp235x_hal::{Timer, Watchdog, block::ImageDef, clocks, entry, pac::Peripherals, usb::UsbBus};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{SerialPort, embedded_io::Write};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);

    let clocks = clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        peripherals.USB,
        peripherals.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut peripherals.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Company")
            .product("Product")
            .serial_number("TEST")])
        .unwrap()
        .device_class(2)
        .build();

    let mut buffer = [0u8; 32];
    let mut writer = buffer.as_mut_slice();
    write!(writer, "{}", "hello").unwrap();

    let mut last_send_time_us = timer.get_counter().ticks();

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 1_000_000 {
            let _ = serial.write(&buffer[0..]);
            let _ = serial.flush();

            let _ = serial.write(b" world\r\n");
            let _ = serial.flush();

            last_send_time_us = current_time_us;
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
