//! Encoder trait for encoding data into bytes.

/// Trait for encoding messages into a byte vector.
pub trait Encoder<T> {
    /// Encodes the given data into a byte vector.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the input data to be encoded.
    ///
    /// # Arguments
    ///
    /// * `data`: A reference to the data of type `Self::Input` to be encoded
    ///
    /// # Returns
    ///
    /// A Result containing the encoded byte vector or an error message.
    fn encode(&mut self, data: &T) -> Result<Vec<u8>, String>;
}

/// A no-op encoder implementation for the unit type `()`.
impl Encoder<()> for () {
    fn encode(&mut self, _data: &Self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}
