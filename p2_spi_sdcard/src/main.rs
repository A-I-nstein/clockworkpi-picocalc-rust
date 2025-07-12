#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
use embedded_hal::spi::MODE_0;
use embedded_hal_bus::spi::CriticalSectionDevice;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use panic_halt as _;
use rp235x_hal::{
    Clock, Sio, Spi, Timer, Watchdog,
    block::ImageDef,
    clocks, entry,
    fugit::HertzU32,
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

    let timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

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
    let spi = Spi::<_, _, _, 8>::new(peripherals.SPI0, (spi_mosi, spi_miso, spi_sck));

    let mut read_speed = 1_000_000;

    let spi = spi.init(
        &mut peripherals.RESETS,
        clocks.peripheral_clock.freq(),
        HertzU32::from_raw(read_speed),
        MODE_0,
    );

    let spi_mutex = Mutex::new(RefCell::new(spi));

    let sd_card_device = CriticalSectionDevice::new(&spi_mutex, spi_cs, timer).unwrap();

    let sd_card = SdCard::new(sd_card_device, timer);

    let volume_mgr = VolumeManager::new(sd_card, DummyTimesource::default());

    let mut last_send_time_us = timer.get_counter().ticks();
    let idx = 0;

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 5_000_000 {
            critical_section::with(|cs| {
                spi_mutex.borrow_ref_mut(cs).set_baudrate(
                    clocks.peripheral_clock.freq(),
                    HertzU32::from_raw(read_speed),
                )
            })
            .raw();
            write!(serial, "Card Read Speed: {}Hz\r\n", read_speed).unwrap();

            volume_mgr.device(|device| match device.num_bytes() {
                Ok(size) => {
                    write!(serial, "Card Size: {} bytes\r\n", size).unwrap();
                    DummyTimesource::default()
                }
                Err(e) => {
                    write!(serial, "Could not read card: {:?}\r\n", e).unwrap();
                    DummyTimesource::default()
                }
            });

            match volume_mgr.open_volume(VolumeIdx(idx)) {
                Ok(volume0) => {
                    serial.write("Volume Open\r\n".as_bytes()).unwrap();
                    let root_dir = volume0.open_root_dir().unwrap();
                    root_dir
                        .iterate_dir(|entry| {
                            write!(serial, "{:?}\r\n", entry.name).unwrap();
                        })
                        .unwrap();
                }
                Err(e) => {
                    write!(serial, "Could not open volume {}: {:?}\r\n", idx, e).unwrap();
                }
            }

            last_send_time_us = current_time_us;
            read_speed += 100_000;
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
