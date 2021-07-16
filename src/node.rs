use crate::{
    snowflake::{Counter, CounterError, NodeId, Timestamp, TimestampError},
    Snowflake,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum NodeError {
    Counter(CounterError),
    SystemTime(Duration),
    Timestamp(TimestampError),
}

impl From<CounterError> for NodeError {
    fn from(error: CounterError) -> Self {
        Self::Counter(error)
    }
}

impl From<TimestampError> for NodeError {
    fn from(error: TimestampError) -> Self {
        Self::Timestamp(error)
    }
}

pub struct Node {
    id: NodeId,
    epoch: u128,
    counter: Counter,
    last_timestamp: Timestamp,
}

impl Node {
    pub fn new(id: NodeId, epoch: u128) -> Self {
        Self {
            id,
            epoch,
            counter: Counter::default(),
            last_timestamp: Timestamp::default(),
        }
    }

    pub fn snowflake(&mut self) -> Result<Snowflake, NodeError> {
        // TODO maybe use tokio::time::SystemTime
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| NodeError::SystemTime(error.duration()))?
            .as_millis();

        if ms < self.epoch {
            //let duration = Duration::from_millis((self.epoch - ms) as _);
            //return Err(Error::SystemTime(duration));
        }

        let timestamp = Timestamp::new((ms - self.epoch) as _)?;

        if timestamp == self.last_timestamp {
            self.counter.increment()?;
        } else {
            self.last_timestamp = timestamp;
            self.counter.reset();
        }

        Ok(Snowflake::new(timestamp, self.id, self.counter))
    }

    #[cfg(feature = "tokio")]
    pub async fn snowflake_retryable(
        &mut self,
        retries: usize,
        sleep: std::time::Duration,
    ) -> Result<Snowflake, NodeError> {
        for _ in 0..retries + 1 {
            match self.snowflake() {
                Ok(snowflake) => return Ok(snowflake),
                Err(NodeError::Counter(_)) => tokio::time::sleep(sleep).await,
                Err(error) => return Err(error),
            }
        }
        todo!()
    }
}
