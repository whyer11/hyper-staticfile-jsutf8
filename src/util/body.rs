use std::{
    io::Error as IoError,
    pin::Pin,
    task::{ready, Context, Poll},
};

use futures_util::stream::Stream;
use hyper::body::{Bytes, Frame};

use super::{FileBytesStream, FileBytesStreamMultiRange, FileBytesStreamRange};

/// Hyper Body implementation for the various types of streams used in static serving.
pub enum Body {
    /// No response body.
    Empty,
    /// Serve a complete file.
    Full(FileBytesStream),
    /// Serve a range from a file.
    Range(FileBytesStreamRange),
    /// Serve multiple ranges from a file.
    MultiRange(FileBytesStreamMultiRange),
}

impl hyper::body::Body for Body {
    type Data = Bytes;
    type Error = IoError;

    fn poll_frame(
        mut self: Pin<&mut Body>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, IoError>>> {
        let opt = ready!(match *self {
            Body::Empty => return Poll::Ready(None),
            Body::Full(ref mut stream) => Pin::new(stream).poll_next(cx),
            Body::Range(ref mut stream) => Pin::new(stream).poll_next(cx),
            Body::MultiRange(ref mut stream) => Pin::new(stream).poll_next(cx),
        });
        Poll::Ready(opt.map(|res| res.map(Frame::data)))
    }
}
