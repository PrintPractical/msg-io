//! Asynchronous Message I/O handler using `futures` traits.
use std::io;

use bytes::{Buf, BytesMut};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::{
    constants::{INITIAL_BUFFER_SIZE, TEMP_BUFFER_SIZE},
    decoder::Decoder,
    encoder::Encoder,
};

/// Asynchronous Message I/O handler using `futures` traits.
pub struct AsyncMessageIo<S, E, D> {
    stream: S,
    encoder: E,
    decoder: D,
    buffer: BytesMut,
}

impl<S, E, D> AsyncMessageIo<S, E, D> {
    /// Creates a new MessageIo instance (Read & Write) with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `AsyncReadExt` and `AsyncWriteExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo`.
    fn new(stream: S, encoder: E, decoder: D) -> Self
    where
        S: Unpin,
        E: Encoder,
        D: Decoder,
    {
        Self {
            stream,
            encoder,
            decoder,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }
}

impl<S, ED> AsyncMessageIo<S, ED, ED> {
    /// Creates a new MessageIo instance for reading and writing with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `AsyncReadExt` and `AsyncWriteExt`.
    /// * `enc_dec`: An encoder/decoder that implements both `Encoder` and `Decoder` traits. Needs to be clone as well.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for reading and writing.
    pub fn new_rw(stream: S, enc_dec: ED) -> Self
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
        ED: Encoder + Decoder + Clone,
    {
        Self::new(stream, enc_dec.clone(), enc_dec)
    }
}

impl<S, D> AsyncMessageIo<S, (), D> {
    /// Creates a new MessageIo instance for reading with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncReadExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for reading.
    pub fn new_reader(stream: S, decoder: D) -> Self
    where
        S: AsyncReadExt + Unpin,
        D: Decoder,
    {
        Self::new(stream, (), decoder)
    }

    /// Reads a message from the stream using the specified decoder.
    /// 
    /// # Type Parameters
    /// 
    /// * `D`: The decoder type that implements the `Decoder` trait for messages of type `M`.
    /// * `M`: The type of the message to be decoded.
    /// 
    /// # Returns
    /// 
    /// The result of the read operation, which is either:
    /// - `Ok(Some(M))`: A successfully decoded message.
    /// - `Ok(None)`: Indicates the end of the stream.
    /// - `Err(io::Error)`: An error occurred during reading or decoding.
    pub async fn read_message<M>(&mut self) -> io::Result<Option<M>>
    where
        D: Decoder<Output = M>,
        S: AsyncReadExt + Unpin,
    {
        loop {
            let mut temp = [0u8; TEMP_BUFFER_SIZE];
            match self.stream.read(&mut temp).await? {
                0 => return Ok(None),
                n => {
                    self.buffer.extend_from_slice(&temp[..n]);
                    match self.decoder.decode(&self.buffer) {
                        crate::decoder::DecoderResult::Continue => continue,
                        crate::decoder::DecoderResult::Done(msg, used) => {
                            self.buffer.advance(used);
                            return Ok(Some(msg));
                        }
                        crate::decoder::DecoderResult::Error(e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                        }
                    }
                }
            }
        }
    }
}

impl<S, E> AsyncMessageIo<S, E, ()> {
    /// Creates a new MessageIo instance for writing with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncWriteExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for writing.
    pub fn new_writer(stream: S, encoder: E) -> Self
    where
        S: AsyncWriteExt + Unpin,
        E: Encoder,
    {
        Self::new(stream, encoder, ())
    }

    /// Writes a message to the stream using the specified encoder.
    /// 
    /// # Type Parameters
    /// 
    /// * `E`: The encoder type that implements the `Encoder` trait for messages of type `M`.
    /// * `M`: The type of the message to be encoded.
    /// 
    /// # Returns
    /// 
    /// The result of the write operation, which is either:
    /// - `Ok(())`: The message was successfully written.
    /// - `Err(io::Error)`: An error occurred during encoding or writing.
    pub async fn write_message<M>(&mut self, message: &M) -> io::Result<()>
    where
        E: Encoder<Input = M>,
        S: AsyncWriteExt + Unpin,
    {
        let encoded =
            self.encoder.encode(message).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.stream.write_all(&encoded).await
    }
}
