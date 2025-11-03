use std::{sync::mpsc::Receiver, thread};

use channel_protocol::channel_protocol;

#[channel_protocol]
trait CounterManager {
    fn get_and_inc(i: i32) -> i32;
    fn inc(i: i32);
    fn dec(i: i32);
    fn reset();
    fn get() -> i32;
}

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
    let (counter_client, counter_manager_rx) = CounterManagerClient::new();
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
