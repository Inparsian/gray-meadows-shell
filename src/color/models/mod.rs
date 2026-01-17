pub mod rgba;
pub mod hsv;
pub mod hsl;
pub mod cmyk;
pub mod oklab;
pub mod oklch;

pub use {
    rgba::{Rgba, LinearRgba},
    hsv::Hsv,
    hsl::Hsl,
    cmyk::Cmyk,
    oklab::Oklab,
    oklch::Oklch
};

pub trait ColorModel {
    /// Gets this color from a hex string.
    fn from_hex(hex: &str) -> Self where Self: Sized;
    
    /// Gets this color from a string representation.
    fn from_string(string: &str) -> Option<Self> where Self: Sized;
    
    /// Gets this color from another color model.
    /// 
    /// This converts the source model into a hex string, then parses that hex string into this model.
    fn from_model<M: ColorModel>(source: M) -> Self where Self: Sized {
        Self::from_hex(&source.into_hex())
    }
    
    /// Converts this color into a hex string.
    fn into_hex(self) -> String;
    
    /// Converts this color into it's string representation.
    fn into_string(self) -> String;
    
    /// Converts this color into a int.
    /// 
    /// Is an alias for hex_to_int(&self.into_hex()).
    fn into_int(self) -> u32 where Self: Sized {
        let hex = self.into_hex();
        super::hex_to_int(&hex)
    }
}