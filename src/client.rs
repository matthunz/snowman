use crate::{http::SnowflakeResponse, node::NodeError, Snowflake};
use http::Uri;
use hyper::{ Body};
use hyper::client::HttpConnector;
use hyper::body::Buf;

pub enum ClientError {
    Http(hyper::Error),
    Json(serde_json::Error),
    Node(NodeError),
}

pub struct Client {
    http: hyper::Client<HttpConnector, Body>,
    uri: Uri,
}

impl Client {
    pub fn new(uri: Uri) -> Self {
        Self {
            http: hyper::Client::new(),
            uri,
        }
    }

    pub async fn snowflake(&self) -> Result<Snowflake, ClientError> {
        let res = self
            .http
            .get(self.uri.clone())
            .await
            .map_err(ClientError::Http)?;

        let buf = hyper::body::aggregate(res)
            .await
            .map_err(ClientError::Http)?;

        let snowflake_res = serde_json::from_reader(buf.reader()).map_err(ClientError::Json)?;

        match snowflake_res {
            SnowflakeResponse::Snowflake(snowflake) => Ok(snowflake),
            SnowflakeResponse::Error(node_error) => Err(ClientError::Node(node_error)),
        }
    }
}
