pub const BYTE_DIVISOR: f64 = 1024.0;

pub fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / BYTE_DIVISOR.powi(3)
}