use futures_intrusive::channel::shared::{self, ChannelSendFuture, Receiver, Sender};
use futures_util::future;
use http::{Request, Response};
use parking_lot::RawMutex;
use snowman::Snowflake;
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use tower_service::Service;

type Message = oneshot::Sender<Snowflake>;

// TODO remove unpin bounds and change variance
pub struct SnowflakeServiceFuture<B> {
    send: Option<Pin<Box<ChannelSendFuture<RawMutex, Message>>>>,
    oneshot: oneshot::Receiver<Snowflake>,
    _marker: PhantomData<B>,
}

impl<B> Future for SnowflakeServiceFuture<B>
where
    B: From<String> + Unpin,
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
            .map_ok(|snowflake| Response::new(B::from(snowflake.id.to_string())))
            .map_err(|_| todo!())
    }
}

pub struct SnowflakeService {
    sender: Sender<Message>,
}

impl<B> Service<Request<B>> for SnowflakeService
where
    B: From<String> + Unpin,
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
