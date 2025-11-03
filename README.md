# Channel Protocol

![crate.io](https://img.shields.io/crates/v/channel-protocol?style=for-the-badge&logo=rust&logoColor=white)

## What is it ?

A procedural macro to generate channel protocol clients.

You can use function oriented communication between threads instead of communicating by sending messages through channels.

This is an abstraction over channels that makes inter-thread communication easier to use and read.

## Example

```rust
use std::{sync::mpsc::Receiver, thread};

use channel_protocol::channel_protocol;

// That's the important bit
#[channel_protocol]
trait CounterManager {
    fn get_and_inc(i: i32) -> i32;
    fn inc(i: i32);
    fn dec(i: i32);
    fn reset();
    fn get() -> i32;
}

// This is an example of how to implement communication with the generated client
fn manager_thread(rx: Receiver<CounterManagerMessage>) {
    let mut counter = 0;
    for message in rx {
        match message {
            CounterManagerMessage::Inc { i } => {
                counter += i;
            }
            CounterManagerMessage::Dec { i } => {
                counter -= i;
            }
            CounterManagerMessage::Reset => {
                counter = 0;
            }
            CounterManagerMessage::Get { return_sender } => {
                return_sender.send(counter).unwrap();
            }
            CounterManagerMessage::GetAndInc { i, return_sender } => {
                return_sender.send(counter).unwrap();
                counter += i;
            }
        }
    }
}

fn main() {
    // Create the client generated from the `channel_protocol` macro
    let (counter_client, counter_manager_rx) = CounterManagerClient::new();

    // Spawn the manager thread that will handle the messages sent by the client
    thread::spawn(|| {
        manager_thread(counter_manager_rx);
    });

    assert_eq!(0, counter_client.get());
    counter_client.inc(2);
    assert_eq!(2, counter_client.get());
    counter_client.dec(1);
    assert_eq!(1, counter_client.get());
    counter_client.reset();
    assert_eq!(0, counter_client.get());
    assert_eq!(0, counter_client.get_and_inc(5));
    assert_eq!(5, counter_client.get());
}

```
