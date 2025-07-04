#![no_std]
#![no_main]

use embedded_hal::{delay::DelayNs, pwm::SetDutyCycle};
use panic_halt as _;
use rp235x_hal::{
    Sio, Timer, Watchdog, block::ImageDef, clocks, entry, gpio::Pins, pac::Peripherals,
    pwm::Slices, usb::UsbBus,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{SerialPort, embedded_io::Write};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000_u32;

const PWM_DIV_INT: u8 = 64;

const fn get_top(freq: f64, div_int: u8) -> u16 {
    let result = 150_000_000. / (freq * div_int as f64);
    result as u16 - 1
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

    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let mut pwm_slices = Slices::new(peripherals.PWM, &mut peripherals.RESETS);

    let pwm = &mut pwm_slices.pwm5;
    pwm.enable();
    pwm.set_div_int(PWM_DIV_INT);
    pwm.channel_a.output_to(pins.gpio26);
    pwm.channel_b.output_to(pins.gpio27);

    let mut last_send_time_us = timer.get_counter().ticks();

    loop {
        let current_time_us = timer.get_counter().ticks();

        if current_time_us - last_send_time_us >= 1_000_000 {
            let top = get_top(100.0, PWM_DIV_INT);
            pwm.set_top(top);
            match pwm.channel_a.set_duty_cycle_percent(50) {
                Ok(()) => {
                    let _ = serial.write(b"Success L\r\n");
                    let _ = serial.flush();
                }
                Err(e) => {
                    let mut buffer = [0u8; 32];
                    let mut writer = buffer.as_mut_slice();
                    write!(writer, "{:?}\r\n", e).unwrap();
                    let _ = serial.write(&buffer[0..]);
                    let _ = serial.flush();
                }
            }
            timer.delay_ms(1000);
            pwm.channel_a.set_duty_cycle(0).unwrap();

            timer.delay_ms(100);

            let top = get_top(1000.0, PWM_DIV_INT);
            pwm.set_top(top);
            match pwm.channel_b.set_duty_cycle_percent(50) {
                Ok(()) => {
                    let _ = serial.write(b"Success R\r\n");
                    let _ = serial.flush();
                }
                Err(e) => {
                    let mut buffer = [0u8; 32];
                    let mut writer = buffer.as_mut_slice();
                    write!(writer, "{:?}\r\n", e).unwrap();
                    let _ = serial.write(&buffer[0..]);
                    let _ = serial.flush();
                }
            }
            timer.delay_ms(500);
            pwm.channel_b.set_duty_cycle(0).unwrap();

            last_send_time_us = current_time_us;
        }

        usb_dev.poll(&mut [&mut serial]);
    }
}
