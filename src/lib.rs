pub mod node;
pub use node::Node;

pub mod snowflake;
pub use snowflake::Snowflake;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

#[cfg(any(feature = "client", feature = "server"))]
mod http {
    use crate::node::NodeError;
    use crate::Snowflake;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SnowflakeResponse {
        Snowflake(Snowflake),
        Error(NodeError),
    }
}
