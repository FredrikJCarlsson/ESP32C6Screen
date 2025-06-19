#![no_std]
#![no_main]

use core::fmt::Display;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use esp_hal::spi::master::Config;
use esp_hal::time::Rate;
use esp_println::println;

use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{Orientation, Rotation},
    Builder,
};

use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    spi::{master::Spi, Mode},
};

use embedded_hal_bus::spi::ExclusiveDevice;

fn init_spi() -> spiDevice {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let sclk = peripherals.GPIO7;
    let miso = peripherals.GPIO5;
    let mosi = peripherals.GPIO6;

    let lcd_dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let lcd_cs = Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default());

    let lcd_rst = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    let back_light = Output::new(peripherals.GPIO22, Level::Low, OutputConfig::default());

    let mut delay_spi = Delay::new();

    // PWM for Backlight
    let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(10)).unwrap();
    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    // connect operator0 to pin
    let mut pwm_pin = mcpwm
        .operator0
        .with_pin_a(back_light, PwmPinConfig::UP_ACTIVE_HIGH);

    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, Rate::from_khz(25))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);
    // 10% duty cycle
    pwm_pin.set_timestamp(10);
    // ###############

    println!("Creating SPI");
    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(20))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_miso(miso)
    .with_mosi(mosi);
    println!("SPI created");

    println!("Creating SPI Device");
    let mut buffer = [0_u8; 512];
    let spi_device = ExclusiveDevice::new(spi, lcd_cs, delay_spi).unwrap();
    let di = SpiInterface::new(spi_device, lcd_dc, &mut buffer);
    println!("SPI Device created");

    return di;
}

fn init_display() -> Display {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let lcd_rst = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    let delay = Delay::new();

    println!("Creating Display");
    let mut display = Builder::new(ST7789, di)
        .reset_pin(lcd_rst)
        .init(&mut delaySpi)
        .unwrap();
    println!("Display created");

    return display;
}

fn start() {
    let spi_device = init_spi();
    let display = init_display();

    println!("Clearing Display");
    display.clear(Rgb565::BLACK).unwrap();

    println!("Setting Orientation");
    display
        .set_orientation(Orientation::new().rotate(Rotation::Deg270))
        .unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);

    Text::with_alignment(
        "Hello world, it works!",
        Point::new(50, 50),
        style,
        Alignment::Left,
    )
    .draw(&mut display)
    .unwrap();
}
