use futures_intrusive::channel::shared;
use snowman::Node;
use tokio::sync::oneshot;
use tokio::task;

pub mod server;

enum Message {
    Snowflake(oneshot::Sender<i64>),
    Close,
}

pub enum Error {}

#[tokio::main]
async fn main() {
    let epoch = 0;
    let node_ids = 0..10;

    let (sender, receiver) = shared::channel(100);

    let nodes = node_ids.map(|id| {
        let receiver = receiver.clone();
        task::spawn(async move {
            let mut node = Node::new(id, epoch);
            loop {
                match receiver.receive().await {
                    Some(Message::Snowflake(oneshot)) => {
                        let snowflake = node.snowflake(5).await.unwrap();
                        oneshot.send(snowflake).ok();
                    }
                    Some(Message::Close) | None => break,
                };
            }
        })
    });

    for handle in nodes {
        handle.await.unwrap();
    }
}
