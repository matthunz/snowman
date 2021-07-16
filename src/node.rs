use crate::{
    snowflake::{Counter, CounterError, NodeId, Timestamp, TimestampError},
    Snowflake,
};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use tokio::time;

pub enum NodeError {
    Counter(CounterError),
    SystemTime(SystemTimeError),
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
        // TODO check id
        Self {
            id,
            epoch,
            counter: Counter::default(),
            last_timestamp: Timestamp::default(),
        }
    }

    pub fn try_snowflake(&mut self) -> Result<Snowflake, NodeError> {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| NodeError::SystemTime(error))?
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

    pub async fn snowflake(&mut self, retries: usize) -> Result<Snowflake, NodeError> {
        for _ in 0..retries + 1 {
            match self.try_snowflake() {
                Ok(snowflake) => return Ok(snowflake),
                Err(NodeError::Counter(_)) => time::sleep(Duration::from_millis(1)).await,
                Err(error) => return Err(error),
            }
        }
        todo!()
    }
}
