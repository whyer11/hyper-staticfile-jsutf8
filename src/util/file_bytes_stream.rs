use futures_util::stream::Stream;
use hyper::body::{Body, Bytes};
use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::prelude::AsyncRead;

const BUF_SIZE: usize = 8 * 1024;

/// Wraps a `tokio::fs::File`, and implements a stream of `Bytes`s.
pub struct FileBytesStream {
    file: File,
    buf: Box<[u8; BUF_SIZE]>,
}

impl FileBytesStream {
    /// Create a new stream from the given file.
    pub fn new(file: File) -> FileBytesStream {
        let buf = Box::new([0; BUF_SIZE]);
        FileBytesStream { file, buf }
    }
}

impl Stream for FileBytesStream {
    type Item = Result<Bytes, IoError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let Self {
            ref mut file,
            ref mut buf,
        } = *self;
        match Pin::new(file).poll_read(cx, &mut buf[..]) {
            Poll::Ready(Ok(0)) => Poll::Ready(None),
            Poll::Ready(Ok(size)) => Poll::Ready(Some(Ok(self.buf[..size].to_owned().into()))),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FileBytesStream {
    /// Create a Hyper `Body` from this stream.
    pub fn into_body(self) -> Body {
        Body::wrap_stream(self)
    }
}
