#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Flex, Input, Level, Output, Pin};
use embassy_rp::spi::{self, Spi};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::{mono_font::ascii::FONT_5X7, mono_font::MonoTextStyle};
use inky_frame_rs::display::color::OctColor;
use inky_frame_rs::display::{Display5in65f, Epd5in65f, IsBusy, HEIGHT};
use inky_frame_rs::shift_register::InkyFrameShiftRegister;

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
    //keeps pico awake by turning on vsts
    let mut vsys = Output::new(pico.PIN_2, Level::High);

    let mut config = spi::Config::default();
    config.frequency = 3_000_000;
    let spi = Spi::new_blocking(pico.SPI0, clk, mosi, miso, config);
    let spi_bus = shared_bus::BusManagerSimple::new(spi);

    let mut shift_register = InkyFrameShiftRegister::new(
        Output::new(pico.PIN_8.degrade(), Level::High),
        Output::new(pico.PIN_9.degrade(), Level::Low),
        Input::new(pico.PIN_10, embassy_rp::gpio::Pull::None),
    );

    let is_busy_bit = shift_register.read_register_bit(7).unwrap();
    let is_busy = shift_register.is_busy();

    // let mut shift_register_clock_pin = Output::new(pico.PIN_8, Level::High);
    // let mut shift_register_latch_pin = Output::new(pico.PIN_9, Level::Low);
    // let shift_register_read_pin = Input::new(pico.PIN_10, embassy_rp::gpio::Pull::None);

    // shift_register_latch_pin.set_low();
    // Timer::after(Duration::from_millis(1)).await;
    // shift_register_latch_pin.set_high();

    // let mut result = 0u8;
    // let mut bits =

    warn!("is busy {}; bit reading is {}", is_busy, is_busy_bit);

    let mut e_ink_display = Display5in65f::default();
    warn!("creating e_ink_device");
    debug!("HII");

    let Ok(mut e_ink_device) = Epd5in65f::new(
        &mut spi_bus.acquire_spi(),
        Output::new(e_ink_cs, Level::High),
        Output::new(e_ink_dc, Level::Low),
        Output::new(e_ink_reset, Level::Low),
        &mut shift_register,
    ) else {
        error!("Failed to create device");
        loop {
            info!("looping around");
            Timer::after(Duration::from_secs(1)).await;
        }
    };
    // let mut e_ink_device = Epd5in65f::new(
    //     &mut spi_bus.acquire_spi(),
    //     Output::new(e_ink_cs, Level::High),
    //     Output::new(e_ink_dc, Level::Low),
    //     Output::new(e_ink_reset, Level::Low),
    //     &mut shift_register,
    // )
    // .unwrap();

    debug!("Clearing the frame");

    match e_ink_device.clear_frame(&mut spi_bus.acquire_spi(), &mut shift_register) {
        Ok(_) => debug!("Successfully cleared the frame"),
        Err(e) => error!("{}", e),
    }

    // debug!("drawing a rectangle");
    // e_ink_display
    //     .fill_solid(
    //         &Rectangle {
    //             top_left: Point::new(0, 0),
    //             size: Size::new(100, 100),
    //         },
    //         OctColor::Blue,
    //     )
    //     .unwrap();

    // let character_style = MonoTextStyle::new(&FONT_5X7, OctColor::Black);

    // match
    let x = Triangle::new(
        Point::new(16, 16 + YOFFSET),
        Point::new(16 + 16, 16 + YOFFSET),
        Point::new(16 + 8, YOFFSET),
    )
    .into_styled(THIN_STROKE);

// match e_ink_display.draw_iter(x.pixels()) {
    //     Ok(_) => debug!("successfully drew pixels"),
    //     Err(_) => debug!("Error drawing pixels"),
    // };
    
    debug!(
        "({}, {})",
        x.bounding_box().center().x,
        x.bounding_box().center().y
    );
    // let pixels = x.pixels();
    // match x.draw(&mut e_ink_display) {
    //     Ok(_) => info!("Successfully drew triangle"),
    //     Err(e) => error!("{}", e),
    // };
    // .into_styled(THIN_STROKE)
    // .draw(&mut e_ink_display) {
    //     Ok(_) => info!("Successfully drew triangle"),
    //     Err(e) => error!("{}", e)
    // };
    // //Draw a triangle.
    // Triangle::new(
    //     Point::new(16, 16 + YOFFSET),
    //     Point::new(16 + 16, 16 + YOFFSET),
    //     Point::new(16 + 8, YOFFSET),
    // )
    // .into_styled(THIN_STROKE)
    // .draw(&mut e_ink_display)
    // .unwrap();

    // // Draw a filled square
    // Rectangle::new(Point::new(52, YOFFSET), Size::new(16, 16))
    //     .into_styled(FILL)
    //     .draw(&mut e_ink_display)
    //     .unwrap();

    // // Draw a circle with a 3px wide stroke.
    // Circle::new(Point::new(88, YOFFSET), 17)
    //     .into_styled(THICK_STROKE)
    //     .draw(&mut e_ink_display)
    //     .unwrap();
    // let width = i32::try_from(e_ink_device.width()).unwrap();
    // let height = i32::try_from(e_ink_device.height()).unwrap();
    // Text::with_alignment(
    //     "Hi Rust",
    //     Point::new(width / 2_i32, height / 2_i32) + Point::new(0, 15),
    //     character_style,
    //     Alignment::Center,
    // )
    // .draw(&mut e_ink_display)
    // .unwrap();

    // e_ink_device
    //     .display_frame(&mut spi_bus.acquire_spi(), &mut shift_register)
    //     .unwrap();
    loop {
        info!("led on!");
        vsys.set_high();
        Timer::after(Duration::from_secs(1)).await;

        info!("led off!");
        vsys.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}
