#![no_std]
#![no_main]

use embedded_hal::{delay::DelayNs, spi::MODE_0};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use panic_halt as _;
use rp235x_hal::{
    Clock, Sio, Spi, Timer, Watchdog,
    block::ImageDef,
    clocks, entry,
    fugit::RateExtU32,
    gpio::{FunctionSpi, Pins},
    pac::Peripherals,
    usb::UsbBus,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{SerialPort, embedded_io::Write};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 50,
            zero_indexed_month: 6,
            zero_indexed_day: 2,
            hours: 7,
            minutes: 48,
            seconds: 35,
        }
    }
}

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

    let mut timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

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

    let spi_cs = pins.gpio17.into_push_pull_output();
    let spi_sck = pins.gpio18.into_function::<FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<FunctionSpi>();
    let spi_miso = pins.gpio16.into_function::<FunctionSpi>();
    let spi_bus = Spi::<_, _, _, 8>::new(peripherals.SPI0, (spi_mosi, spi_miso, spi_sck));

    let spi = spi_bus.init(
        &mut peripherals.RESETS,
        clocks.peripheral_clock.freq(),
        200.kHz(),
        MODE_0,
    );

    let spi = ExclusiveDevice::new(spi, spi_cs, timer).unwrap();

    timer.delay_ms(100);

    let sdcard = SdCard::new(spi, timer);
    let volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());

    let mut last_send_time_us = timer.get_counter().ticks();

    let mut read_flag = false;

    loop {
        let current_time_us = timer.get_counter().ticks();

        if !read_flag && current_time_us - last_send_time_us >= 1_000_000 {
            volume_mgr.device(|device| match device.num_bytes() {
                Ok(size) => {
                    let mut buffer = [0u8; 64];
                    let mut writer = buffer.as_mut_slice();
                    write!(writer, "Card Size: {:?}\r\n", size).unwrap();
                    serial.write(&buffer[0..]).unwrap();
                    DummyTimesource::default()
                }
                Err(e) => {
                    let mut buffer = [0u8; 64];
                    let mut writer = buffer.as_mut_slice();
                    write!(writer, "Could not read card: {:?}\r\n", e).unwrap();
                    serial.write(&buffer[0..]).unwrap();
                    DummyTimesource::default()
                }
            });

            volume_mgr.device(|device| {
                device.spi(|spi| {
                    spi.bus_mut()
                        .set_baudrate(clocks.peripheral_clock.freq(), 16.MHz());
                    DummyTimesource::default()
                })
            });

            match volume_mgr.open_volume(VolumeIdx(0)) {
                Ok(_volume0) => {
                    serial.write("Volume Open\r\n".as_bytes()).unwrap();
                }
                Err(e) => {
                    let mut buffer = [0u8; 64];
                    let mut writer = buffer.as_mut_slice();
                    write!(writer, "Could not open volume: {:?}\r\n", e).unwrap();
                    serial.write(&buffer[0..]).unwrap();
                }
            }

            last_send_time_us = current_time_us;
            read_flag = true;
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
