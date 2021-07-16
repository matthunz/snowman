use snowman::server;
use snowman::snowflake::NodeId;
use snowman::Node;
use std::time::Duration;
use tokio::task;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let epoch = 1420070400000;
    let node_ids = 0..10;

    let (server, receiver) = server::bind(&addr, 100);

    let nodes: Vec<_> = node_ids
        .map(|id| {
            let node_id = NodeId::new(id).map_err(|_| ()).unwrap();
            let receiver = receiver.clone();
            task::spawn(async move {
                let mut node = Node::new(node_id, epoch);
                loop {
                    match receiver.receive().await {
                        Some(oneshot) => {
                            let result =
                                node.snowflake_retryable(5, Duration::from_millis(1)).await;
                            oneshot.send(result).ok();
                        }
                        None => break,
                    };
                }
            })
        })
        .collect();

    server.await.unwrap();

    for handle in nodes.into_iter() {
        handle.await.unwrap();
    }
}
