use crate::node::NodeError;
use crate::Snowflake;
use futures_intrusive::channel::shared::{self, ChannelSendFuture, Receiver, Sender};
use futures_util::future;
use http::{Request, Response, StatusCode};
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use parking_lot::RawMutex;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use tower_service::Service;

type Message = oneshot::Sender<Result<Snowflake, NodeError>>;

// TODO remove unpin bounds and change variance
pub struct SnowflakeServiceFuture<B> {
    send: Option<Pin<Box<ChannelSendFuture<RawMutex, Message>>>>,
    oneshot: oneshot::Receiver<Result<Snowflake, NodeError>>,
    _marker: PhantomData<B>,
}

impl<B> Future for SnowflakeServiceFuture<B>
where
    B: From<String> + Default + Unpin,
{
    type Output = Result<Response<B>, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let me = self.get_mut();
        if let Some(ref mut send) = me.send {
            if send.as_mut().poll(cx).is_ready() {
                me.send = None;
            } else {
                return Poll::Pending;
            }
        }
        Pin::new(&mut me.oneshot)
            .poll(cx)
            .map_ok(|result| {
                let (response, status) = match result {
                    Ok(snowflake) => (SnowflakeResponse::Snowflake(snowflake), StatusCode::OK),
                    Err(error) => (
                        SnowflakeResponse::Error(error),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                };
                serde_json::to_string(&response)
                    .ok()
                    .and_then(|json| Response::builder().status(status).body(B::from(json)).ok())
                    .unwrap_or_else(|| {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(B::default())
                            .unwrap()
                    })
            })
            .map_err(|_| todo!())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnowflakeResponse {
    Snowflake(Snowflake),
    Error(NodeError),
}

pub struct SnowflakeService {
    sender: Sender<Message>,
}

impl<B> Service<Request<B>> for SnowflakeService
where
    B: From<String> + Default + Unpin,
{
    type Response = Response<B>;
    type Error = Infallible;
    type Future = SnowflakeServiceFuture<B>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<B>) -> Self::Future {
        let (tx, rx) = oneshot::channel();
        let send = Box::pin(self.sender.send(tx));
        SnowflakeServiceFuture {
            send: Some(send),
            oneshot: rx,
            _marker: PhantomData,
        }
    }
}

pub struct SnowflakeMakeService {
    sender: Sender<Message>,
}

impl SnowflakeMakeService {
    pub fn with_capacity(capacity: usize) -> (Self, Receiver<Message>) {
        let (sender, receiver) = shared::channel(capacity);
        (Self { sender }, receiver)
    }
}

impl<T> Service<T> for SnowflakeMakeService {
    type Response = SnowflakeService;
    type Error = Infallible;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        let service = SnowflakeService {
            sender: self.sender.clone(),
        };
        future::ready(Ok(service))
    }
}

pub fn bind(
    addr: &SocketAddr,
    capacity: usize,
) -> (
    Server<AddrIncoming, SnowflakeMakeService>,
    Receiver<Message>,
) {
    let (make_svc, receiver) = SnowflakeMakeService::with_capacity(capacity);
    (Server::bind(addr).serve(make_svc), receiver)
}
