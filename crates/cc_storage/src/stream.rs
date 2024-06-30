use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::io::{AsyncRead as FAsyncRead, IoSliceMut};
use pin_project::pin_project;
use std::path::PathBuf;
use std::{fmt, io};

use futures::{Future, Stream};
use tokio::{fs::File, io::AsyncReadExt};

use crate::error::ObjectResult;
use bytes::Bytes;
use opendal::Operator;

const DEFAULT_BUFFER_SIZE: usize = 2048;

/// The callback function triggered every time a chunck of the source file is read
/// in the buffer.
///
/// # Arguments
/// * `u64`: The total length of the buffer (or size of the file if created from a `PathBuf`)
/// * `u64`: The total number of bytes read so far
/// * `u64`: The number of bytes read in the current chunck
type CallbackFn = dyn FnMut(&str, u64, u64, u64) + Sync + Send + 'static;

/// A `futures::Stream` implementation that can be used to track uploads
/// ```
pub struct TrackableBodyStream<I: AsyncReadExt + Unpin> {
    input: I,
    file_size: u64,
    cur_read: u64,
    key: String,
    callback: Option<Box<CallbackFn>>,
    buffer_size: usize,
}

impl TryFrom<PathBuf> for TrackableBodyStream<File> {
    type Error = std::io::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file_size = std::fs::metadata(value.clone())?.len();
        let file = futures::executor::block_on(tokio::fs::File::open(value))?;
        Ok(Self {
            input: file,
            file_size,
            cur_read: 0,
            key: String::new(),
            callback: None,
            buffer_size: DEFAULT_BUFFER_SIZE,
        })
    }
}

impl<I: AsyncReadExt + Unpin + Send + Sync + 'static> TrackableBodyStream<I> {
    /// Sets the callback method
    pub fn set_callback(
        &mut self,
        key: &str,
        callback: impl FnMut(&str, u64, u64, u64) + Sync + Send + 'static,
    ) {
        self.key = key.to_string();
        self.callback = Some(Box::new(callback));
    }
}

impl<I: AsyncReadExt + Unpin> Stream for TrackableBodyStream<I> {
    type Item = Result<Bytes, Box<dyn std::error::Error + Sync + std::marker::Send + 'static>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut_self = self.get_mut();
        let mut buf = Vec::with_capacity(mut_self.buffer_size);

        match Future::poll(Box::pin(mut_self.input.read_buf(&mut buf)).as_mut(), cx) {
            Poll::Ready(res) => {
                if res.is_err() {
                    return Poll::Ready(Some(Err(Box::new(res.err().unwrap()))));
                }
                let read_op = res.unwrap();
                if read_op == 0 {
                    return Poll::Ready(None);
                }
                mut_self.cur_read += read_op as u64;
                //buf.resize(read_op, 0u8);
                if mut_self.callback.is_some() {
                    mut_self.callback.as_mut().unwrap()(
                        mut_self.key.as_str(),
                        mut_self.file_size,
                        mut_self.cur_read,
                        read_op as u64,
                    );
                }
                Poll::Ready(Some(Ok(Bytes::from(Vec::from(&buf[0..read_op])))))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            (self.file_size - self.cur_read) as usize,
            Some(self.file_size as usize),
        )
    }
}

pub type BoxedStreamingUploader = Box<StreamingUploader>;

/// Store multiple parts in a map, and concatenate them on finish.
pub struct StreamingUploader {
    op: Operator,
    path: String,
    buffer: Vec<Bytes>,
}
impl StreamingUploader {
    pub fn new(op: Operator, path: String) -> Self {
        Self {
            op,
            path,
            buffer: vec![],
        }
    }

    pub async fn write_bytes(&mut self, data: Bytes) -> ObjectResult<()> {
        self.buffer.push(data);
        Ok(())
    }

    pub async fn finish(self: Box<Self>) -> ObjectResult<()> {
        self.op.write(&self.path, self.buffer).await?;

        Ok(())
    }
}

/// Reader for the `report_progress` method.
#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct StreamDownloader<St, F> {
    #[pin]
    inner: St,
    callback: F,
    bytes_read: usize,
}

impl<St, F> fmt::Debug for StreamDownloader<St, F>
where
    St: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamDownloader")
            .field("stream", &self.inner)
            .finish()
    }
}

pub trait AsyncReadProgressExt {
    fn report_progress<F>(self, callback: F) -> StreamDownloader<Self, F>
    where
        Self: Sized,
        F: FnMut(usize),
    {
        let bytes_read = 0;
        StreamDownloader {
            inner: self,
            callback,
            bytes_read,
        }
    }
}

impl<R: FAsyncRead + ?Sized> AsyncReadProgressExt for R {}

impl<St, F> FAsyncRead for StreamDownloader<St, F>
where
    St: FAsyncRead,
    F: FnMut(usize),
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        match this.inner.poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(bytes_read)) => {
                *this.bytes_read += bytes_read;
                (this.callback)(*this.bytes_read);
                Poll::Ready(Ok(bytes_read))
            }
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        match this.inner.poll_read_vectored(cx, bufs) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(bytes_read)) => {
                *this.bytes_read += bytes_read;
                (this.callback)(*this.bytes_read);
                Poll::Ready(Ok(bytes_read))
            }
        }
    }
}
