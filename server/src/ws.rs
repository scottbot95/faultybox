use futures::{Sink, SinkExt, StreamExt};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use axum::body::Bytes;
use axum::extract::{ws, FromRequestParts};
use axum::extract::ws::{DefaultOnFailedUpgrade, OnFailedUpgrade};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use futures::Stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
// use serde_json::Error;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct WebSocketUpgrade<S, R, F=DefaultOnFailedUpgrade> {
    upgrade: ws::WebSocketUpgrade<F>,
    _marker: PhantomData<fn() -> (S, R, F)>,
}

impl<S, R, B> FromRequestParts<B> for WebSocketUpgrade<S, R>
where
    B: Send + Sync
{
    type Rejection = <ws::WebSocketUpgrade as FromRequestParts<B>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &B) -> Result<Self, Self::Rejection> {
        let upgrade = <ws::WebSocketUpgrade as FromRequestParts<B>>::from_request_parts(parts, state).await?;
        Ok(Self {
            upgrade,
            _marker: PhantomData,
        })
    }
}

impl<S, R, F: OnFailedUpgrade> WebSocketUpgrade<S, R, F> {
    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    ///
    /// This is analagous to [`axum::extract::ws::WebSocketUpgrade::on_upgrade`].
    pub fn on_upgrade<Fun, Fut>(self, callback: Fun) -> Response
    where
        Fun: FnOnce(WebSocket<S, R>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        S: Send,
        R: Send,
    {

        self.upgrade
            .on_upgrade(|socket| async move {
                let socket = WebSocket {
                    socket,
                    _marker: PhantomData,
                };
                callback(socket).await
            })
    }

    pub fn on_failed_upgrade<C>(self, callback: C) -> WebSocketUpgrade<S, R, C>
    where
        C: OnFailedUpgrade,
        S: Send,
        R: Send,
    {

        let upgrade = self.upgrade
            .on_failed_upgrade(callback);
        WebSocketUpgrade {
            upgrade,
            _marker: PhantomData,
        }
    }

    /// Apply a transformation to the inner [`axum::extract::ws::WebSocketUpgrade`].
    ///
    /// This can be used to apply configuration.
    pub fn map<T, T2>(self, f: T) -> WebSocketUpgrade<S, R, T2>
    where
        T: FnOnce(ws::WebSocketUpgrade<F>) -> ws::WebSocketUpgrade<T2>,
    {
        WebSocketUpgrade {
            upgrade: f(self.upgrade),
            _marker: PhantomData,
        }
    }

    /// Get the inner axum [`axum::extract::ws::WebSocketUpgrade`].
    pub fn into_inner(self) -> ws::WebSocketUpgrade<F> {
        self.upgrade
    }
}

impl<S, R, F> fmt::Debug for WebSocketUpgrade<S, R, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebSocketUpgrade")
            .field("upgrade", &self.upgrade)
            .finish()
    }
}

pub struct WebSocket<S, R> {
    socket: ws::WebSocket,
    _marker: PhantomData<fn() -> (S, R)>,
}

impl<S, R> WebSocket<S, R> {
    pub async fn recv(&mut self) -> Option<Result<Message<R>, Error>>
    where
        R: DeserializeOwned,
    {
        self.next().await
    }

    pub async fn send(&mut self, msg: Message<S>) -> Result<(), Error>
    where
        S: Serialize,
    {
        SinkExt::send(self, msg).await
    }

    pub async fn send_item(&mut self, item: S) -> Result<(), Error>
    where
        S: Serialize,
    {
        SinkExt::send(self, Message::Item(item)).await
    }

    pub async fn close(&mut self) -> Result<(), axum::Error> {
        self.socket.close().await
    }

    pub fn into_inner(self) -> ws::WebSocket {
        self.socket
    }
}

impl<S, R> Stream for WebSocket<S, R>
where
    R: DeserializeOwned,
{
    type Item = Result<Message<R>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let msg = futures::ready!(Pin::new(&mut self.socket)
            .poll_next(cx)?);

        if let Some(msg) = msg {
            let msg = match msg {
                ws::Message::Text(msg) => Bytes::from(msg),
                ws::Message::Binary(bytes) => bytes,
                ws::Message::Close(frame) => {
                    return Poll::Ready(Some(Ok(Message::Close(frame))));
                }
                ws::Message::Ping(buf) => {
                    return Poll::Ready(Some(Ok(Message::Ping(buf))));
                }
                ws::Message::Pong(buf) => {
                    return Poll::Ready(Some(Ok(Message::Pong(buf))));
                }
            };

            let msg = serde_json::from_slice(&msg)
                .map(Message::Item)
                .map_err(Into::into);
            Poll::Ready(Some(msg))
        } else {
            Poll::Ready(None)
        }
    }
}

impl<S, R> Sink<Message<S>> for WebSocket<S, R>
where
    S: Serialize,
{
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.socket).poll_ready(cx).map_err(From::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message<S>) -> Result<(), Self::Error> {
        let msg = match item {
            Message::Item(buf) => ws::Message::Binary(serde_json::to_vec(&buf)?.into()),
            // Message::Item(buf) => ws::Message::Text(serde_json::to_string(&buf)?.into()),
            Message::Ping(buf) => ws::Message::Ping(buf),
            Message::Pong(buf) => ws::Message::Pong(buf),
            Message::Close(frame) => ws::Message::Close(frame),
        };

        Pin::new(&mut self.socket)
            .start_send(msg)
            .map_err(From::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.socket).poll_flush(cx).map_err(From::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.socket).poll_close(cx).map_err(From::from)
    }
}

/// A WebSocket message contain a value of a known type.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message<T> {
    /// An item of type `T`.
    Item(T),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping(Bytes),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong(Bytes),
    /// A close message with the optional close frame.
    Close(Option<ws::CloseFrame>),
}