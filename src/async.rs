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
pub struct MessageIo<S> {
    stream: S,
    buffer: BytesMut,
}

impl<S> MessageIo<S> {
    /// Creates a new MessageIo instance (Read & Write) with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `AsyncReadExt` and `AsyncWriteExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo`.
    pub fn new(stream: S) -> Self
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    /// Creates a new MessageIo instance for reading with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncReadExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for reading.
    pub fn new_reader(stream: S) -> Self
    where
        S: AsyncReadExt + Unpin,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
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
    pub async fn read_message<D, M>(&mut self) -> io::Result<Option<M>>
    where
        D: Decoder<M>,
        S: AsyncReadExt + Unpin,
    {
        loop {
            let mut temp = [0u8; TEMP_BUFFER_SIZE];
            match self.stream.read(&mut temp).await? {
                0 => return Ok(None),
                n => {
                    self.buffer.extend_from_slice(&temp[..n]);
                    match D::decode(&self.buffer) {
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

    /// Creates a new MessageIo instance for writing with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncWriteExt`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for writing.
    pub fn new_writer(stream: S) -> Self
    where
        S: AsyncWriteExt + Unpin,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
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
    pub async fn write_message<E, M>(&mut self, message: &M) -> io::Result<()>
    where
        E: Encoder<M>,
        S: AsyncWriteExt + Unpin,
    {
        let encoded =
            E::encode(message).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.stream.write_all(&encoded).await
    }
}
