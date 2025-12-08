pub enum DecoderResult<T> {
    Continue,
    Done(T, usize),
    Error(String),
}

pub trait Decoder<T> {
    fn decode(data: &[u8]) -> DecoderResult<T>;
}