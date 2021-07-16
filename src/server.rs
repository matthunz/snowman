use crate::{Error, Message};
use futures_intrusive::channel::shared::{ChannelSendFuture, Sender};
use http::{Request, Response};
use parking_lot::RawMutex;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use tower_service::Service;

// TODO remove unpin bounds and change variance
pub struct SnowflakeServiceFuture<B> {
    send: Option<Pin<Box<ChannelSendFuture<RawMutex, Message>>>>,
    oneshot: oneshot::Receiver<i64>,
    _marker: PhantomData<B>,
}

impl<B> Future for SnowflakeServiceFuture<B>
where
    B: From<String> + Unpin,
{
    type Output = Result<Response<B>, Error>;

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
            .map_ok(|snowflake| Response::new(B::from(snowflake.to_string())))
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
    type Error = Error;
    type Future = SnowflakeServiceFuture<B>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: Request<B>) -> Self::Future {
        let (tx, rx) = oneshot::channel();
        let send = Box::pin(self.sender.send(Message::Snowflake(tx)));
        SnowflakeServiceFuture {
            send: Some(send),
            oneshot: rx,
            _marker: PhantomData,
        }
    }
}
