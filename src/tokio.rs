//! Asynchronous Message I/O handler using `tokio` traits.
use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::r#async::MessageIo;

// A wrapper around the asynchronous MessageIo to work with Tokio streams.
pub struct MessageTokio;

impl MessageTokio {
    /// Creates a new MessageIo instance (Read & Write) with the given Tokio stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements both `AsyncRead` and `AsyncWrite`.
    /// 
    /// # Returns
    /// 
    /// A new async instance of `MessageIo`.
    pub fn new<S>(stream: S) -> MessageIo<Compat<S>>
    where
        S: TokioAsyncRead + TokioAsyncWrite + Unpin,
    {
        MessageIo::new(stream.compat_write())
    }

    /// Creates a new MessageIo instance for reading with the given Tokio stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncRead`.
    /// 
    /// # Returns
    /// 
    /// A new async instance of `MessageIo` for reading.
    pub fn new_reader<S>(stream: S) -> MessageIo<Compat<S>>
    where
        S: TokioAsyncRead + Unpin,
    {
        MessageIo::new_reader(stream.compat())
    }

    /// Creates a new MessageIo instance for writing with the given Tokio stream.
    /// 
    /// # Arguments
    /// 
    /// * `stream`: An asynchronous stream that implements `AsyncWrite`.
    /// 
    /// # Returns
    /// 
    /// A new async instance of `MessageIo` for writing.
    pub fn new_writer<S>(stream: S) -> MessageIo<Compat<S>>
    where
        S: TokioAsyncWrite + Unpin,
    {
        MessageIo::new_writer(stream.compat_write())
    }
}
