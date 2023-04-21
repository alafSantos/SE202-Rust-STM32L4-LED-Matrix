use crate::{Color, Image};
use core::convert::Infallible;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::Pixel;

impl From<Rgb888> for Color {
    fn from(color: Rgb888) -> Self {
        Self {
            r: (color.r()),
            g: (color.g()),
            b: (color.b()),
        }
    }
}

impl OriginDimensions for Image {
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}

impl DrawTarget for Image {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0 && coord.x < 8 && coord.y >= 0 && coord.y < 8 {
                let index: u32 = (coord.x + coord.y * 8) as u32;
                self[(0, index as usize)] = color.into();
            }
        }
        return Ok(());
    }
}
