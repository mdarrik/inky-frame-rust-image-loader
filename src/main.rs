#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use core::convert::Infallible;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Flex, Level, Output};
use embassy_rp::pac::pwm::Pwm;
use embassy_rp::pac::{pwm, PWM};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::spi;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi::{ExclusiveDevice, SpiBusFlush, SpiBusRead, SpiBusWrite};
use embedded_io::asynch::Write;

use static_cell::StaticCell;
// use embedded_graphics::pixelcolor::BinaryColor;
// use embedded_graphics::prelude::*;
// use embedded_graphics::primitives::{
//     Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
// };
// use embedded_graphics::text::{Alignment, Text};
// use embedded_graphics::{mono_font::ascii::FONT_5X7, mono_font::MonoTextStyle};

// use inky_frame_rs::display::color::OctColor;
// use inky_frame_rs::display::{Display5in65f, Epd5in65f};

use {defmt_rtt as _, panic_probe as _};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

// static THIN_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyle::with_stroke(OctColor::Black, 1);
// static THICK_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyle::with_stroke(OctColor::Black, 3);
// static BORDER_STROKE: PrimitiveStyle<OctColor> = PrimitiveStyleBuilder::new()
//     .stroke_color(OctColor::Black)
//     .stroke_width(3)
//     .stroke_alignment(StrokeAlignment::Inside)
//     .build();

// static FILL: PrimitiveStyle<OctColor> = PrimitiveStyle::with_fill(OctColor::Black);

// static YOFFSET: i32 = 14;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Timer::after(Duration::from_secs(10)).await;
    // warn!("program start");

    #[cfg(feature = "include_firmware")]
    // Include the WiFi firmware and Country Locale Matrix (CLM) blobs.
    let fw = include_bytes!("../wifi-firmware/43439A0.bin");
    #[cfg(feature = "include_firmware")]
    let clm = include_bytes!("../wifi-firmware/43439A0_clm.bin");
    #[cfg(not(feature = "include_firmware"))]
    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs-cli download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs-cli download 43439A0.clm_blob --format bin --chip RP2040 --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    #[cfg(not(feature = "include_firmware"))]
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pico = embassy_rp::init(Default::default());
    let pwr = Output::new(pico.PIN_23, Level::Low);
    let cs = Output::new(pico.PIN_25, Level::High);
    let clk = Output::new(pico.PIN_29, Level::Low);
    let mut dio = Flex::new(pico.PIN_24);
    dio.set_low();
    dio.set_as_output();
    let bus = MySpi { clk, dio };
    let spi = ExclusiveDevice::new(bus, cs);

    let state = singleton!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;
    // let mut led_pin = Output::new(pico.PIN_25, Level::Low);
    // let e_ink_reset = pico.PIN_27;
    // let e_ink_cs = pico.PIN_17;
    // let e_ink_dc = pico.PIN_28;
    // let miso = pico.PIN_16;
    // let clk = pico.PIN_18;
    // let mosi = pico.PIN_19;
    // Output::new(pico.PIN_2, Level::High);
    // let led_A = (pico.PIN_11, Level::High);
    // let mut config = spi::Config::default();
    // config.frequency = 3_000_000;
    // let spi = Spi::new_blocking(pico.SPI0, clk, mosi, miso, config);
    // let spi_bus = shared_bus::BusManagerSimple::new(spi);

    // let mut e_ink_display = Display5in65f::default();
    // let mut e_ink_device = Epd5in65f::new(
    //     &mut spi_bus.acquire_spi(),
    //     Output::new(e_ink_cs, Level::Low),
    //     Output::new(e_ink_dc, Level::Low),
    //     Output::new(e_ink_reset, Level::Low),
    //     &mut embassy_time::Delay,
    // )
    // .unwrap();

    // e_ink_device
    //     .clear_frame(&mut spi_bus.acquire_spi(), &mut embassy_time::Delay)
    //     .unwrap();
    // // let mut e_ink_epd = Epd5in65f::new(&mut spi_bus.acquire_spi(), Output::new(e_ink_cs, Level::High), Output::, e_ink_dc.into(), e_ink_reset.into(), delay);

    // let character_style = MonoTextStyle::new(&FONT_5X7, OctColor::Black);

    // // Draw a triangle.
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
    //     .display_frame(&mut spi_bus.acquire_spi(), &mut Delay)
    //     .unwrap();

    // println!("Hello, world!");
    loop {
        info!("high");
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;
        warn!("low");
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}

struct MySpi {
    /// SPI clock
    clk: Output<'static, PIN_29>,

    /// 4 signals, all in one!!
    /// - SPI MISO
    /// - SPI MOSI
    /// - IRQ
    /// - strap to set to gSPI mode on boot.
    dio: Flex<'static, PIN_24>,
}

impl ErrorType for MySpi {
    type Error = Infallible;
}

impl SpiBusFlush for MySpi {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SpiBusRead<u32> for MySpi {
    async fn read(&mut self, words: &mut [u32]) -> Result<(), Self::Error> {
        self.dio.set_as_input();
        for word in words {
            let mut w = 0;
            for _ in 0..32 {
                w = w << 1;

                // rising edge, sample data
                if self.dio.is_high() {
                    w |= 0x01;
                }
                self.clk.set_high();

                // falling edge
                self.clk.set_low();
            }
            *word = w
        }

        Ok(())
    }
}

impl SpiBusWrite<u32> for MySpi {
    async fn write(&mut self, words: &[u32]) -> Result<(), Self::Error> {
        self.dio.set_as_output();
        for word in words {
            let mut word = *word;
            for _ in 0..32 {
                // falling edge, setup data
                self.clk.set_low();
                if word & 0x8000_0000 == 0 {
                    self.dio.set_low();
                } else {
                    self.dio.set_high();
                }

                // rising edge
                self.clk.set_high();

                word = word << 1;
            }
        }
        self.clk.set_low();

        self.dio.set_as_input();
        Ok(())
    }
}
