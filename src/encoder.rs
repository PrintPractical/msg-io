//! Encoder trait for encoding data into bytes.

/// Trait for encoding messages into a byte vector.
pub trait Encoder<T> {
    /// Encodes the given data into a byte vector.
    /// 
    /// # Arguments
    /// 
    /// * `data`: A reference to the data of type `T` to be encoded
    /// 
    /// # Returns
    /// 
    /// A Result containing the encoded byte vector or an error message.
    fn encode(data: &T) -> Result<Vec<u8>, String>;
}
