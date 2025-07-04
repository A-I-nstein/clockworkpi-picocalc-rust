#![no_std]
#![no_main]

use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb666, prelude::RgbColor};
use embedded_hal::{digital::OutputPin, spi::MODE_0};
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::{Builder, interface::SpiInterface, models::ILI9486Rgb666};
use panic_halt as _;
use rp235x_hal::{
    Clock, Sio, Spi, Timer, Watchdog,
    block::ImageDef,
    clocks, entry,
    fugit::RateExtU32,
    gpio::{FunctionSpi, Pins},
    pac::Peripherals,
};

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

    let mut timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let spi_sck = pins.gpio10.into_function::<FunctionSpi>();
    let spi_mosi = pins.gpio11.into_function::<FunctionSpi>();
    let spi_miso = pins.gpio12.into_function::<FunctionSpi>();

    let spi_bus = Spi::<_, _, _, 8>::new(peripherals.SPI1, (spi_mosi, spi_miso, spi_sck));

    let spi = spi_bus.init(
        &mut peripherals.RESETS,
        clocks.peripheral_clock.freq(),
        300.kHz(),
        MODE_0,
    );

    let mut buffer = [0_u8; 512];
    let dc = pins.gpio14.into_push_pull_output();
    let mut spi_cs = pins.gpio13.into_push_pull_output();
    let mut rst = pins.gpio15.into_push_pull_output();
    spi_cs.set_high().unwrap();
    rst.set_high().unwrap();

    let spi_device = ExclusiveDevice::new_no_delay(spi, spi_cs).unwrap();
    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    let mut display = Builder::new(ILI9486Rgb666, di)
        .reset_pin(rst)
        .init(&mut timer)
        .unwrap();

    display.clear(Rgb666::GREEN).unwrap();

    loop {}
}
