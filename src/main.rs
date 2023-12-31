#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pin};
use embassy_rp::spi::{self, Spi};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use inky_frame_rs::display::color::OctColor;
use inky_frame_rs::display::{InkyFrame5_7, InkyFrameDisplay, IsBusy, HEIGHT, WIDTH};
use inky_frame_rs::shift_register::InkyFrameShiftRegister;
use tinybmp::Bmp;

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

    warn!("is busy {}; bit reading is {}", is_busy, is_busy_bit);

    let mut e_ink_display = InkyFrameDisplay::default();
    warn!("creating e_ink_device");

    let Ok(mut e_ink_device) = InkyFrame5_7::new(
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

    debug!("Clearing the frame");

    e_ink_device.set_background_color(OctColor::Black);
    match e_ink_device.clear_frame(&mut spi_bus.acquire_spi(), &mut shift_register) {
        Ok(_) => debug!("Successfully cleared the frame"),
        Err(e) => error!("{}", e),
    }

    let bmp_image = include_bytes!("../assets/party-corgi-happy-2.bmp");

    let bmp = Bmp::<Rgb888>::from_slice(bmp_image).unwrap();

    let style: embedded_graphics::primitives::PrimitiveStyle<OctColor> =
        embedded_graphics::primitives::PrimitiveStyleBuilder::new()
            .stroke_color(OctColor::Black)
            .stroke_width(5)
            .fill_color(OctColor::Black)
            .build();

    let rectangle = &embedded_graphics::primitives::Rectangle {
        top_left: Point::new(0, 0),
        size: Size {
            width: WIDTH,
            height: HEIGHT,
        },
    }
    .into_styled(style);

    e_ink_display
        .draw_iter(rectangle.pixels())
        .unwrap_or_else(|_| {
            defmt::error!("Error drawing rectangle");
            ()
        });

    match Image::new(
        &bmp,
        Point::new(((WIDTH / 2) as i32) - 50, ((HEIGHT / 2) as i32) - 50),
    )
    .draw(&mut e_ink_display.color_converted())
    {
        Ok(_) => info!("Drew image to buffer"),
        Err(_) => error!("Failed to draw image"),
    }

    Text::with_alignment(
        "Hello!",
        e_ink_display.bounding_box().bottom_right().unwrap() - Point::new((WIDTH / 2) as i32, 25),
        embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            OctColor::HiZ,
        ),
        embedded_graphics::text::Alignment::Center,
    )
    .draw(&mut e_ink_display)
    .unwrap();

    match e_ink_device.update_and_display_frame(
        &mut spi_bus.acquire_spi(),
        &mut shift_register,
        e_ink_display.buffer(),
    ) {
        Ok(_) => info!("Drew image to screen"),
        Err(_) => error!("Failed to draw image to screen"),
    };
    vsys.set_low();

    loop {}
}
