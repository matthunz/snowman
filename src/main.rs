use hyper::Server;
use server::SnowflakeMakeService;
use snowman::snowflake::NodeId;
use snowman::Node;
use tokio::task;

pub mod server;

#[tokio::main]
async fn main() {
    let addr = "localhost:8080".parse().unwrap();
    let epoch = 0;
    let node_ids = 0..10;

    let (make_svc, receiver) = SnowflakeMakeService::with_capacity(100);

    let nodes = node_ids.map(|id| {
        let node_id = NodeId::new(id).map_err(|_| ()).unwrap();
        let receiver = receiver.clone();
        task::spawn(async move {
            let mut node = Node::new(node_id, epoch);
            loop {
                match receiver.receive().await {
                    Some(oneshot) => {
                        let snowflake = node.snowflake(5).await.map_err(|_| ()).unwrap();
                        oneshot.send(snowflake).ok();
                    }
                    None => break,
                };
            }
        })
    });

    Server::bind(&addr).serve(make_svc).await.unwrap();

    for handle in nodes {
        handle.await.unwrap();
    }
}
