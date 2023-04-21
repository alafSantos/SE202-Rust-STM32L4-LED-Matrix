#![no_std]
pub mod image;
pub mod matrix;
pub use image::{Color, Image};
pub mod gamma;
pub use gamma::gamma_correct;
