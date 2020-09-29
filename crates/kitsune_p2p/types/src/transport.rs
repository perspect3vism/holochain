//! A collection of definitions related to remote communication.

use futures::{future::FutureExt, sink::SinkExt, stream::StreamExt};

/// Error related to remote communication.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TransportError {
    /// GhostError.
    #[error(transparent)]
    GhostError(#[from] ghost_actor::GhostError),

    /// Unspecified error.
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl TransportError {
    /// promote a custom error type to a TransportError
    pub fn other(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::Other(e.into())
    }
}

impl From<String> for TransportError {
    fn from(s: String) -> Self {
        #[derive(Debug, thiserror::Error)]
        struct OtherError(String);
        impl std::fmt::Display for OtherError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        TransportError::other(OtherError(s))
    }
}

impl From<&str> for TransportError {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<TransportError> for () {
    fn from(_: TransportError) {}
}

/// Result type for remote communication.
pub type TransportResult<T> = Result<T, TransportError>;

/// Receiver side of the channel
pub type TransportChannelRead =
    Box<dyn futures::stream::Stream<Item = Vec<u8>> + Send + Unpin + 'static>;

/// Extension trait for channel readers
pub trait TransportChannelReadExt {
    /// Read the stream to close into a single byte vec.
    fn read_to_end(self)
        -> ghost_actor::dependencies::must_future::MustBoxFuture<'static, Vec<u8>>;
}

impl<T: futures::stream::Stream<Item = Vec<u8>> + Send + Unpin + 'static> TransportChannelReadExt
    for T
{
    fn read_to_end(
        self,
    ) -> ghost_actor::dependencies::must_future::MustBoxFuture<'static, Vec<u8>> {
        async move {
            self.fold(Vec::new(), |mut acc, x| async move {
                acc.extend_from_slice(&x);
                acc
            })
            .await
        }
        .boxed()
        .into()
    }
}

/// Sender side of the channel
pub type TransportChannelWrite =
    Box<dyn futures::sink::Sink<Vec<u8>, Error = TransportError> + Send + Unpin + 'static>;

/// Extension trait for channel writers
pub trait TransportChannelWriteExt {
    /// Write all data and close channel
    fn write_and_close<'a>(
        &'a mut self,
        data: Vec<u8>,
    ) -> ghost_actor::dependencies::must_future::MustBoxFuture<'a, TransportResult<()>>;
}

impl<T: futures::sink::Sink<Vec<u8>, Error = TransportError> + Send + Unpin + 'static>
    TransportChannelWriteExt for T
{
    fn write_and_close<'a>(
        &'a mut self,
        data: Vec<u8>,
    ) -> ghost_actor::dependencies::must_future::MustBoxFuture<'a, TransportResult<()>> {
        async move {
            self.send(data).await?;
            self.close().await?;
            Ok(())
        }
        .boxed()
        .into()
    }
}

/// Tuple sent through TransportIncomingChannel Sender/Receiver.
pub type TransportIncomingChannel = (url2::Url2, TransportChannelWrite, TransportChannelRead);

/// Send new incoming channel data.
pub type TransportIncomingChannelSender = futures::channel::mpsc::Sender<TransportIncomingChannel>;

/// Receiving a new incoming channel connection.
pub type TransportIncomingChannelReceiver =
    futures::channel::mpsc::Receiver<TransportIncomingChannel>;

ghost_actor::ghost_chan! {
    /// Represents a transport binding for establishing connections.
    /// This api was designed mainly around supporting the QUIC transport.
    /// It should be applicable to other transports, but with some assumptions:
    /// - Keep alive logic should be handled internally.
    /// - Transport encryption is handled internally.
    /// - See light-weight comments below on `create_channel` api.
    pub chan TransportListener<TransportError> {
        /// Retrieve the current url (address) this listener is bound to.
        fn bound_url() -> url2::Url2;

        /// Attempt to establish an outgoing channel to a remote.
        /// Channels are expected to be very light-weight.
        /// This API was designed around QUIC bi-streams.
        /// If your low-level channels are not light-weight, consider
        /// implementing pooling/multiplex virtual channels to
        /// make this api light weight.
        fn create_channel(url: url2::Url2) -> (
            url2::Url2,
            TransportChannelWrite,
            TransportChannelRead,
        );
    }
}

/// Extension trait for additional methods on TransportListenerSenders
pub trait TransportListenerSenderExt {
    /// Make a request using a single channel open/close.
    fn request(
        &self,
        url: url2::Url2,
        data: Vec<u8>,
    ) -> ghost_actor::dependencies::must_future::MustBoxFuture<'static, TransportResult<Vec<u8>>>;
}

impl<T: TransportListenerSender> TransportListenerSenderExt for T {
    fn request(
        &self,
        url: url2::Url2,
        data: Vec<u8>,
    ) -> ghost_actor::dependencies::must_future::MustBoxFuture<'static, TransportResult<Vec<u8>>>
    {
        let fut = self.create_channel(url);
        async move {
            let (_url, mut write, read) = fut.await?;
            write.write_and_close(data).await?;
            Ok(read.read_to_end().await)
        }
        .boxed()
        .into()
    }
}
