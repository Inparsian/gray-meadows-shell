pub mod rgba;
pub mod hsv;
pub mod hsl;
pub mod cmyk;
pub mod oklab;
pub mod oklch;

pub use {
    rgba::Rgba,
    hsv::Hsv,
    hsl::Hsl,
    cmyk::Cmyk,
    oklab::Oklab,
    oklch::Oklch
};

pub trait ColorModel {
    /// Gets this color from RGBA.
    fn from_rgba(rgba: Rgba) -> Self where Self: Sized;

    /// Converts this color into RGBA.
    fn into_rgba(self) -> Rgba where Self: Sized;
    
    /// Gets this color from a string representation.
    fn from_string(string: &str) -> Option<Self> where Self: Sized;
    
    /// Converts this color into it's string representation.
    fn into_string(self) -> String;
    
    /// Gets this color from another color model.
    /// 
    /// This converts the source model into RGBA, then converts that RGBA into this model.
    fn from_model<M: ColorModel>(source: M) -> Self where Self: Sized {
        Self::from_rgba(source.into_rgba())
    }

    /// Gets this color from a hex string.
    ///
    /// Is an alias for `Self::from_rgba(Rgba::from_hex(hex))`.
    fn from_hex(hex: &str) -> Self where Self: Sized {
        Self::from_rgba(Rgba::from_hex(hex))
    }

    /// Converts this color into a hex string.
    /// 
    /// Is an alias for `self.into_rgba().into_hex()`.
    fn into_hex(self) -> String where Self: Sized {
        self.into_rgba().into_hex()
    }

    /// Converts this color into a int.
    /// 
    /// Is an alias for `hex_to_int(&self.into_hex())`.
    fn into_int(self) -> u32 where Self: Sized {
        let hex = self.into_hex();
        super::hex_to_int(&hex)
    }
}