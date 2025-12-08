use std::io::{self, Read, Write};

use bytes::{Buf, BytesMut};

use crate::{
    decoder::{Decoder, DecoderResult},
    encoder::Encoder,
};

const INITIAL_BUFFER_SIZE: usize = 1024;
const TEMP_BUFFER_SIZE: usize = 1024;

pub struct MessageIo<S> {
    stream: S,
    buffer: BytesMut,
}

impl<S> MessageIo<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    pub fn new_reader(stream: S) -> Self
    where
        S: Read,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    pub fn read_message<D, M>(&mut self) -> io::Result<Option<M>>
    where
        D: Decoder<M>,
        S: Read,
    {
        loop {
            let mut temp = [0u8; TEMP_BUFFER_SIZE];
            match self.stream.read(&mut temp)? {
                0 => return Ok(None),
                n => {
                    self.buffer.extend_from_slice(&temp[..n]);
                    match D::decode(&self.buffer) {
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

    pub fn new_writer(stream: S) -> Self
    where
        S: Write,
    {
        Self {
            stream,
            buffer: BytesMut::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }

    pub fn write_message<E, M>(&mut self, msg: &M) -> io::Result<()>
    where
        E: Encoder<M>,
        S: Write,
    {
        let encoded = E::encode(msg).map_err(|e| io::Error::other(e))?;
        self.stream.write_all(&encoded)?;
        Ok(())
    }
}
