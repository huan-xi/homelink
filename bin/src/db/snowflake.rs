use snowdon::{Epoch, Generator, Layout, Snowflake};
use std::cell::RefCell;
use std::sync::Arc;

fn machine_id() -> u64 {
    // This is where you would implement your machine ID
    0
}

// We make our structure hidden, as this is an implementation detail that most
// users of our custom ID won't need
#[derive(Debug)]
#[doc(hidden)]
pub struct MySnowflakeParams;

impl Layout for MySnowflakeParams {
    fn construct_snowflake(timestamp: u64, sequence_number: u64) -> u64 {
        assert!(
            !Self::exceeds_timestamp(timestamp) && !Self::exceeds_sequence_number(sequence_number)
        );
        (timestamp << 22) | (machine_id() << 12) | sequence_number
    }
    fn timestamp(input: u64) -> u64 {
        input >> 22
    }
    fn exceeds_timestamp(input: u64) -> bool {
        input >= (1 << 42)
    }
    fn sequence_number(input: u64) -> u64 {
        input & ((1 << 12) - 1)
    }
    fn exceeds_sequence_number(input: u64) -> bool {
        input >= (1 << 12)
    }
    fn is_valid_snowflake(input: u64) -> bool {
        // Our snowflake format doesn't have any constant parts that we could
        // validate here
        true
    }
}

impl Epoch for MySnowflakeParams {
    fn millis_since_unix() -> u64 {
        // Our epoch for this example is the first millisecond of 2015
        1420070400000
    }
}

// Define our snowflake and generator types
pub type MySnowflake = Snowflake<MySnowflakeParams, MySnowflakeParams>;

pub struct MySnowflakeGenerator {
    inner: Arc<Generator<MySnowflakeParams, MySnowflakeParams>>,
}

impl Default for MySnowflakeGenerator {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl MySnowflakeGenerator {
    pub fn next_id(&self) -> i64 {
        self.inner.clone().generate().unwrap().into_inner() as i64
        // self.0.generate().unwrap().into_inner()
    }
}

// pub type MySnowflakeGenerator = ;

#[test]
pub fn test() {
    let mut a = MySnowflakeGenerator::default();
    println!("{:?}", a.next_id());
}
