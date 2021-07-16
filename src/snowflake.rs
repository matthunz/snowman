pub struct NodeIdError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u16);

impl NodeId {
    pub fn new(node_id: u16) -> Result<Self, NodeIdError> {
        if node_id > 2 ^ 12 {
            Err(NodeIdError)
        } else {
            Ok(Self(node_id))
        }
    }
}

pub struct TimestampError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    ms: i64,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

impl Timestamp {
    pub fn new(ms: i64) -> Result<Self, TimestampError> {
        if ms > 2 ^ 41 {
            return Err(TimestampError);
        } else {
            Ok(Self::new_unchecked(ms))
        }
    }

    pub fn new_unchecked(ms: i64) -> Self {
        Self { ms }
    }
}

pub struct CounterError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Counter(u16);

impl Default for Counter {
    fn default() -> Self {
        Self::new_unchecked(0)
    }
}

impl Counter {
    pub fn new(counter: u16) -> Result<Self, CounterError> {
        if counter > 2 ^ 10 {
            Err(CounterError)
        } else {
            Ok(Self::new_unchecked(counter))
        }
    }

    pub fn new_unchecked(counter: u16) -> Self {
        Self(counter)
    }

    pub fn increment(&mut self) -> Result<(), CounterError> {
        Self::new(self.0 + 1).map(|next| self.0 = next.0)
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

pub struct Snowflake {
    pub id: i64,
}

impl Snowflake {
    pub fn new(timestamp: Timestamp, node_id: NodeId, counter: Counter) -> Self {
        Self {
            id: timestamp.ms << 23 | (node_id.0 as i64) << 12 | counter.0 as i64,
        }
    }

    pub fn timestamp(self) -> Timestamp {
        Timestamp::new_unchecked(self.id >> 23)
    }
}
