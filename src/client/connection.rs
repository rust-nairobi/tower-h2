use {Body, RecvBody};
use super::Background;
use flush::Flush;

use bytes::IntoBuf;
use futures::{Future, Poll};
use futures::future::Executor;
use h2;
use h2::client::{self, Client};
use http::{self, Request, Response};
use tower::Service;
use tokio_io::{AsyncRead, AsyncWrite};

use std::marker::PhantomData;

/// Exposes a request/response API on an h2 client connection..
pub struct Connection<T, E, S>
where S: Body,
{
    client: Client<S::Data>,
    executor: E,
    _p: PhantomData<(T, S)>,
}

/// Drives the sending of a request (and its body) until a response is received (i.e. the
/// initial HEADERS or RESET frames sent from the remote).
///
/// This is necessary because, for instance, the remote server may not respond until the
/// request body is fully sent.
pub struct ResponseFuture {
    inner: Inner,
}

/// ResponseFuture inner
enum Inner {
    /// Inner response future
    Inner(client::ResponseFuture),

    /// Failed to send the request
    Error(Option<Error>),
}

/// Errors produced by client `Connection` calls.
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    Inner(h2::Error),
    Spawn,
}

// ===== impl Connection =====

impl<T, E, S> Connection<T, E, S>
where S: Body,
      S::Data: IntoBuf + 'static,
      E: Executor<Background<T, S>>,
      T: AsyncRead + AsyncWrite,
{
    /// Builds Connection on an H2 client connection.
    pub fn new(client: Client<S::Data>, executor: E) -> Self {
        let _p = PhantomData;

        Connection {
            client,
            executor,
            _p,
        }
    }
}

impl<T, E, S> Clone for Connection<T, E, S>
where S: Body,
      E: Clone,
{
    fn clone(&self) -> Self {
        Connection {
            client: self.client.clone(),
            executor: self.executor.clone(),
            _p: PhantomData,
        }
    }
}

impl<T, E, S> Service for Connection<T, E, S>
where S: Body + 'static,
      S::Data: IntoBuf + 'static,
      E: Executor<Background<T, S>>,
      T: AsyncRead + AsyncWrite,
{
    type Request = Request<S>;
    type Response = Response<RecvBody>;
    type Error = Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.client.poll_ready()
            .map_err(Into::into)
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        trace!("request: {} {}", request.method(), request.uri());

        // Split the request from the body
        let (parts, body) = request.into_parts();
        let request = http::Request::from_parts(parts, ());

        // If there is no body, then there is no point spawning a task to flush
        // it.
        let end_of_stream = body.is_end_stream();

        // Initiate the H2 request
        let res = self.client.send_request(request, end_of_stream);

        let (response, send_body) = match res {
            Ok(success) => success,
            Err(e) => {
                let e = Error { kind: Kind::Inner(e) };
                let inner = Inner::Error(Some(e));
                return ResponseFuture { inner };
            }
        };

        if !end_of_stream {
            let flush = Flush::new(body, send_body);
            let res = self.executor.execute(Background::flush(flush));

            if let Err(_) = res {
                let e = Error { kind: Kind::Spawn };
                let inner = Inner::Error(Some(e));
                return ResponseFuture { inner };
            }
        }

        ResponseFuture { inner: Inner::Inner(response) }
    }
}

// ===== impl ResponseFuture =====

impl Future for ResponseFuture {
    type Item = Response<RecvBody>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::Inner::*;

        match self.inner {
            Inner(ref mut fut) => {
                let response = try_ready!(fut.poll());

                let (parts, body) = response.into_parts();
                let body = RecvBody::new(body);

                Ok(Response::from_parts(parts, body).into())
            }
            Error(ref mut e) => {
                return Err(e.take().unwrap());
            }
        }
    }
}

// ===== impl Error =====

impl Error {
    pub fn reason(&self) -> Option<h2::Reason> {
        match self.kind {
            Kind::Inner(ref h2) => h2.reason(),
            _ => None,
        }
    }
}

impl From<h2::Error> for Error {
    fn from(src: h2::Error) -> Self {
        Error { kind: Kind::Inner(src) }
    }
}

impl From<h2::Reason> for Error {
    fn from(src: h2::Reason) -> Self {
        h2::Error::from(src).into()
    }
}
