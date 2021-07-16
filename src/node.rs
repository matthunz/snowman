use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{ time};

pub struct Node {
    id: u16,
    epoch: u128,
    counter: u16,
    last_timestamp: u64,
}

impl Node {
    pub fn new(id: u16, epoch: u128) -> Self {
        Self {
            id,
            epoch,
            counter: 0,
            last_timestamp: 0,
        }
    }

    pub fn try_snowflake(&mut self) -> Option<i64> {
        let timestamp = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - self.epoch) as u64;

        if timestamp == self.last_timestamp {
            if self.counter > 2 ^ 10 {
                return None;
            }
            self.counter += 1;
        } else {
            self.last_timestamp = timestamp;
            self.counter = 0;
        }

        Some(0)
    }

    pub async fn snowflake(&mut self, retries: usize) -> Option<i64> {
        for _ in 0..retries + 1 {
            match self.try_snowflake() {
                Some(snowflake) => return Some(snowflake),
                None => time::sleep(Duration::from_millis(1)).await,
            }
        }
        None
    }
}