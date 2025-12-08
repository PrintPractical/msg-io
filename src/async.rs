use std::io;

use bytes::{Buf, BytesMut};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::{constants::{INITIAL_BUFFER_SIZE, TEMP_BUFFER_SIZE}, decoder::Decoder, encoder::Encoder};

pub struct MessageIo<S>
{
    stream: S,
    buffer: BytesMut,
}

impl<S> MessageIo<S>{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    pub fn new_reader(stream: S) -> Self
    where
        S: AsyncReadExt + Unpin,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

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

    pub fn new_writer(stream: S) -> Self
    where
        S: AsyncWriteExt + Unpin,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    pub async fn write_message<E, M>(&mut self, message: &M) -> io::Result<()>
    where
        E: Encoder<M>,
        S: AsyncWriteExt + Unpin,
    {
        let encoded = E::encode(message).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.stream.write_all(&encoded).await
    }
}