use futures_intrusive::channel::shared::{self, Sender};
use tokio::sync::oneshot;
use tokio::task;

mod node;
pub use node::Node;

enum Message {
    Snowflake(oneshot::Sender<i64>),
    Close,
}

async fn handle(sender: Sender<Message>) {
    let (tx, rx) = oneshot::channel();
    sender.send(Message::Snowflake(tx)).await.ok();
    let snowflake = rx.await;
}

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
