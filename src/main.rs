#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pac::{PWM, pwm};
use embassy_rp::pac::pwm::Pwm;
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::{mono_font::ascii::FONT_5X7, mono_font::MonoTextStyle};

use inky_frame_rs::display::color::OctColor;
use inky_frame_rs::display::{Display5in65f, Epd5in65f};

use {defmt_rtt as _, panic_probe as _};

static THIN_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyle::with_stroke(OctColor::Black, 1);
static THICK_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyle::with_stroke(OctColor::Black, 3);
static BORDER_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyleBuilder::new()
    .stroke_color(OctColor::Black)
    .stroke_width(3)
    .stroke_alignment(StrokeAlignment::Inside)
    .build();

static FILL: PrimitiveStyle<OctColor> = PrimitiveStyle::with_fill(OctColor::Black);

static YOFFSET: i32 = 14;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let pico = embassy_rp::init(Default::default());
    let e_ink_reset = pico.PIN_27;
    let e_ink_cs = pico.PIN_17;
    let e_ink_dc = pico.PIN_28;
    let miso = pico.PIN_16;
    let clk = pico.PIN_18;
    let mosi = pico.PIN_19;
    Output::new(pico.PIN_2, Level::High);
    let led_A = (pico.PIN_11, Level::High);
    let mut config = spi::Config::default();
    config.frequency = 3_000_000;
    let spi = Spi::new_blocking(pico.SPI0, clk, mosi, miso, config);
    let spi_bus = shared_bus::BusManagerSimple::new(spi);

    let mut e_ink_display = Display5in65f::default();
    let mut e_ink_device = Epd5in65f::new(
        &mut spi_bus.acquire_spi(),
        Output::new(e_ink_cs, Level::Low),
        Output::new(e_ink_dc, Level::Low),
        Output::new(e_ink_reset, Level::Low),
        &mut embassy_time::Delay,
    )
    .unwrap();

    e_ink_device
        .clear_frame(&mut spi_bus.acquire_spi(), &mut embassy_time::Delay)
        .unwrap();
    // let mut e_ink_epd = Epd5in65f::new(&mut spi_bus.acquire_spi(), Output::new(e_ink_cs, Level::High), Output::, e_ink_dc.into(), e_ink_reset.into(), delay);

    let character_style = MonoTextStyle::new(&FONT_5X7, OctColor::Black);

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + YOFFSET),
        Point::new(16 + 16, 16 + YOFFSET),
        Point::new(16 + 8, YOFFSET),
    )
    .into_styled(THIN_STROKE)
    .draw(&mut e_ink_display)
    .unwrap();

    // Draw a filled square
    Rectangle::new(Point::new(52, YOFFSET), Size::new(16, 16))
        .into_styled(FILL)
        .draw(&mut e_ink_display)
        .unwrap();

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, YOFFSET), 17)
        .into_styled(THICK_STROKE)
        .draw(&mut e_ink_display)
        .unwrap();
    let width = i32::try_from(e_ink_device.width()).unwrap();
    let height = i32::try_from(e_ink_device.height()).unwrap();
    Text::with_alignment(
        "Hi Rust",
        Point::new(width / 2_i32, height / 2_i32) + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut e_ink_display)
    .unwrap();

    e_ink_device
        .display_frame(&mut spi_bus.acquire_spi(), &mut Delay)
        .unwrap();

    println!("Hello, world!");
    loop {}
}
