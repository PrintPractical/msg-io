//! Decoder result types and traits.

/// Represents the result of a decoding operation.
/// - `Continue`: Indicates that more data is needed to decode a complete message.
/// - `Done(T, usize)`: Indicates that a message of type `T` has been successfully decoded,
///   along with the number of bytes used from the input data.
/// - `Error(String)`: Indicates that an error occurred during decoding, with an error message.
pub enum DecoderResult<T> {
    Continue,
    Done(T, usize),
    Error(String),
}

/// Trait for decoding messages from a byte slice.
pub trait Decoder<T> {
    /// Decodes a message from the given byte slice.
    /// 
    /// # Arguments
    /// 
    /// * `data`: A byte slice containing the data to decode.
    /// 
    /// # Returns
    /// 
    /// A DecoderResult indicating the outcome of the decoding operation.
    fn decode(data: &[u8]) -> DecoderResult<T>;
}
