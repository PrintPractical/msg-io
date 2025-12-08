use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{r#async::MessageIo};

pub struct MessageTokio;

impl MessageTokio
{
    pub fn new<S>(stream: S) -> MessageIo<Compat<S>>
    where 
        S: TokioAsyncRead + TokioAsyncWrite + Unpin,
    {
        MessageIo::new(stream.compat_write())
    }

    pub fn new_reader<S>(stream: S) -> MessageIo<Compat<S>>
    where
        S: TokioAsyncRead + Unpin,
    {
        MessageIo::new_reader(stream.compat())
    }
    
    pub fn new_writer<S>(stream: S) -> MessageIo<Compat<S>>
    where
        S: TokioAsyncWrite + Unpin,
    {
        MessageIo::new_writer(stream.compat_write())
    }
}