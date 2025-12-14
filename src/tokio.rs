//! Asynchronous Message I/O handler using `tokio` traits.
use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{r#async::AsyncMessageIo, decoder::Decoder, encoder::Encoder};

// A wrapper around the asynchronous MessageIo to work with Tokio streams.
pub struct MessageTokio;

impl MessageTokio {
    /// Creates a new MessageIo instance (Read & Write) with the given Tokio stream.
    ///
    /// # Arguments
    ///
    /// * `stream`: An asynchronous stream that implements both `AsyncRead` and `AsyncWrite`.
    /// * `enc_dec`: A struct that implements both `Encoder` and `Decoder` traits. Needs to be clone as well.
    ///
    /// # Returns
    ///
    /// A new async instance of `MessageIo`.
    pub fn new_rw<S, ED>(stream: S, enc_dec: ED) -> AsyncMessageIo<Compat<S>, ED, ED>
    where
        S: TokioAsyncRead + TokioAsyncWrite + Unpin,
        ED: Encoder + Decoder + Clone,
    {
        AsyncMessageIo::new_rw(stream.compat_write(), enc_dec)
    }

    /// Creates a new MessageIo instance for reading with the given Tokio stream.
    ///
    /// # Arguments
    ///
    /// * `stream`: An asynchronous stream that implements `AsyncRead`.
    /// * `decoder`: A decoder that implements the `Decoder` trait.
    ///
    /// # Returns
    ///
    /// A new async instance of `MessageIo` for reading.
    pub fn new_reader<S, D>(stream: S, decoder: D) -> AsyncMessageIo<Compat<S>, (), D>
    where
        S: TokioAsyncRead + Unpin,
        D: Decoder,
    {
        AsyncMessageIo::new_reader(stream.compat(), decoder)
    }

    /// Creates a new MessageIo instance for writing with the given Tokio stream.
    ///
    /// # Arguments
    ///
    /// * `stream`: An asynchronous stream that implements `AsyncWrite`.
    /// * `encoder`: An encoder that implements the `Encoder` trait.
    ///
    /// # Returns
    ///
    /// A new async instance of `MessageIo` for writing.
    pub fn new_writer<S, E>(stream: S, encoder: E) -> AsyncMessageIo<Compat<S>, E, ()>
    where
        S: TokioAsyncWrite + Unpin,
        E: Encoder,
    {
        AsyncMessageIo::new_writer(stream.compat_write(), encoder)
    }
}
