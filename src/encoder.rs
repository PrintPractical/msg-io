pub trait Encoder<T> {
    fn encode(data: &T) -> Result<Vec<u8>, String>;
}
