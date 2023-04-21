use crate::gamma;
use core::ops::Mul;

#[repr(C)] // Force Rust to use a C compatible representation for Color
#[derive(Copy, Clone, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub fn gamma_correct(&self) -> Self {
        Self {
            r: gamma::gamma_correct(self.r),
            g: gamma::gamma_correct(self.g),
            b: gamma::gamma_correct(self.b),
        }
    }
}

// In order to check the range
fn range_dealer(x: usize) -> u8 {
    if x <= 255 {
        return x as u8;
    } else {
        return 255;
    }
}

impl core::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, value: f32) -> Self::Output {
        let r = (self.r as f32) * value;
        let g = (self.g as f32) * value;
        let b = (self.b as f32) * value;

        return Color {
            r: range_dealer(r as usize),
            g: range_dealer(g as usize),
            b: range_dealer(b as usize),
        };
    }
}

impl core::ops::Div<f32> for Color {
    type Output = Self;

    fn div(self, value: f32) -> Self::Output {
        return self.mul(1.0 / value);
    }
}

#[repr(transparent)] // to ensure that it keeps the same representation as its unique element.
pub struct Image([Color; 64]);

impl Image {
    pub fn new_solid(color: Color) -> Self {
        let image = [color; 64];
        return Image(image);
    }

    pub fn row(&self, row: usize) -> &[Color] {
        return &self.0[(8 * row)..(8 * (row + 1))];
    }

    pub fn gradient(color: Color) -> Self {
        let mut grad = Image::new_solid(color);
        for i in 0..8 {
            for j in 0..8 {
                // Each pixel should receive the reference color divided by (1 + row * row + col)
                grad[(i, j)] = color / (1.0 + (i * i + j) as f32);
            }
        }
        return grad;
    }
}

impl Default for Image {
    fn default() -> Self {
        return Image::new_solid(Color::default());
    }
}

impl core::ops::Index<(usize, usize)> for Image {
    type Output = Color;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        /* Since Image has 64 positions in one dimention,
           we convert a ideal matrix ixj to an one dimention array by doing 8*i + j */
        return &self.0[8 * index.0 + index.1];
    }
}

impl core::ops::IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        /* Since Image has 64 positions in one dimention,
           we convert a ideal matrix ixj to an one dimention array by doing 8*i + j */
        return &mut self.0[8 * index.0 + index.1];
    }
}

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe {
            return core::mem::transmute(self);
        }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe {
            return core::mem::transmute(self);
        }
    }
}
