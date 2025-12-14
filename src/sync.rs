//! Synchronous Message I/O handler using `std::io` traits.
use std::io::{self, Read, Write};

use bytes::{Buf, BytesMut};

use crate::{
    decoder::{Decoder, DecoderResult},
    encoder::Encoder,
};

const INITIAL_BUFFER_SIZE: usize = 1024;
const TEMP_BUFFER_SIZE: usize = 1024;

/// Message I/O handler using `std::io` traits.
pub struct MessageIo<S, E, D> {
    stream: S,
    encoder: E,
    decoder: D,
    buffer: BytesMut,
}

impl<S, E, D> MessageIo<S, E, D> {
    /// Creates a new MessageIo instance (Read & Write) with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `Read` and `Write`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo`.
    fn new(stream: S, encoder: E, decoder: D) -> Self {
        Self {
            stream,
            encoder,
            decoder,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }
}

impl<S, ED> MessageIo<S, ED, ED> {
    /// Creates a new MessageIo instance for reading and writing with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `Read` and `Write`.
    /// * `enc_dec`: An encoder/decoder that implements both `Encoder` and `Decoder` traits. Needs to be clone as well.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for reading and writing.
    pub fn new_rw<T>(stream: S, enc_dec: ED) -> Self
    where
        S: Read + Write,
        ED: Encoder + Decoder + Clone,
    {
        Self::new(stream, enc_dec.clone(), enc_dec)
    }
}

impl<S, D> MessageIo<S, (), D> {
    /// Creates a new MessageIo instance for reading with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `Read`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for reading.
    pub fn new_reader(stream: S, decoder: D) -> Self
    where
        S: Read,
        D: Decoder,
    {
        Self {
            stream,
            encoder: (),
            decoder,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    /// Reads a message from the stream using the specified decoder.
    /// 
    /// # Type Parameters
    /// 
    /// * `D`: The decoder type that implements the `Decoder` trait.
    /// * `M`: The message type to be decoded.
    /// 
    /// # Returns
    /// 
    /// A result containing an optional message of type `M`.
    pub fn read_message<M>(&mut self) -> io::Result<Option<M>>
    where
        D: Decoder<Output = M>,
        S: Read,
    {
        loop {
            let mut temp = [0u8; TEMP_BUFFER_SIZE];
            match self.stream.read(&mut temp)? {
                0 => return Ok(None),
                n => {
                    self.buffer.extend_from_slice(&temp[..n]);
                    match self.decoder.decode(&self.buffer) {
                        DecoderResult::Continue => continue,
                        DecoderResult::Done(msg, used) => {
                            self.buffer.advance(used);
                            return Ok(Some(msg));
                        }
                        DecoderResult::Error(e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
                        }
                    }
                }
            }
        }
    }
}

impl<S,E> MessageIo<S, E, ()> {
    /// Creates a new MessageIo instance for writing with the given stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `Write`.
    /// 
    /// # Returns
    /// 
    /// A new instance of `MessageIo` for writing.
    pub fn new_writer(stream: S, encoder: E) -> Self
    where
        S: Write,
        E: Encoder,
    {
        Self {
            stream,
            encoder,
            decoder: (),
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    /// Writes a message to the stream using the specified encoder.
    /// 
    /// # Type Parameters
    /// 
    /// * `E`: The encoder type that implements the `Encoder` trait.
    /// * `M`: The type of the message to be encoded.
    /// 
    /// # Returns
    /// 
    /// The result of the write operation, which is either:
    /// - `Ok(())`: The message was successfully written.
    /// - `Err(io::Error)`: An error occurred during encoding or writing.
    pub fn write_message<M>(&mut self, msg: &M) -> io::Result<()>
    where
        E: Encoder<Input = M>,
        S: Write,
    {
        let encoded = self.encoder.encode(msg).map_err(|e| io::Error::other(e))?;
        self.stream.write_all(&encoded)?;
        Ok(())
    }
}
