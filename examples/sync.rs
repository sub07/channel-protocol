use std::{sync::mpsc::Receiver, thread, time::Duration};

use channel_protocol::channel_protocol;

#[channel_protocol]
trait CounterManager {
    fn get_and_inc(i: i32) -> i32;
    fn inc_and_mul(add: i32, mul: i32) -> i32;
    fn inc(i: i32);
    fn dec(i: i32);
    fn reset();
    fn get() -> i32;
}

#[channel_protocol]
trait CounterOutgoing {
    fn reached_10();
    fn multiple_of_5(val: i32);
}

fn manager_thread(
    counter_outgoing_client: CounterOutgoingClient,
    rx: Receiver<CounterManagerMessage>,
) {
    let mut prev_counter = 0;
    let mut counter = 0;
    for message in rx {
        prev_counter = counter;
        match message {
            CounterManagerMessage::Inc(IncParamMessage { i }) => {
                counter += i;
            }
            CounterManagerMessage::Dec(DecParamMessage { i }) => {
                counter -= i;
            }
            CounterManagerMessage::Reset => {
                counter = 0;
            }
            CounterManagerMessage::Get(ret) => {
                ret.send(counter).unwrap();
            }
            CounterManagerMessage::GetAndInc(GetAndIncParamMessage { i }, ret) => {
                ret.send(counter).unwrap();
                counter += i;
            }
            CounterManagerMessage::IncAndMul(IncAndMulParamMessage { add, mul }, ret) => {
                counter += add;
                counter *= mul;
                ret.send(counter).unwrap();
            }
        }
        if prev_counter != counter {
            if counter == 10 {
                counter_outgoing_client.reached_10();
            }
            if counter % 5 == 0 {
                counter_outgoing_client.multiple_of_5(counter);
            }
        }
    }
}

fn main() {
    let (counter_client, counter_manager_rx) = CounterManagerClient::new();
    let (counter_outgoing_client, counter_outgoing_rx) = CounterOutgoingClient::new();
    thread::spawn(|| {
        manager_thread(counter_outgoing_client, counter_manager_rx);
    });

    thread::spawn(|| {
        for message in counter_outgoing_rx {
            match message {
                CounterOutgoingMessage::Reached10 => {
                    println!("Counter reached 10!");
                }
                CounterOutgoingMessage::MultipleOf5(MultipleOf5ParamMessage { val }) => {
                    println!("Counter is multiple of 5: {val}");
                }
            }
        }
    });

    assert_eq!(0, counter_client.get()); // This should trigger the "multiple of 5" message
    counter_client.inc(2);
    assert_eq!(2, counter_client.get());
    counter_client.dec(1);
    assert_eq!(1, counter_client.get());
    counter_client.reset();
    assert_eq!(0, counter_client.get());
    assert_eq!(0, counter_client.get_and_inc(5)); // This should trigger the "multiple of 5" message
    assert_eq!(5, counter_client.get());
    counter_client.inc(4);
    assert_eq!(9, counter_client.get());
    counter_client.inc(1); // This should trigger the "reached 10" and "multiple of 5" message
    assert_eq!(10, counter_client.get());
    counter_client.inc(5); // This should trigger the "multiple of 5" message
    assert_eq!(15, counter_client.get());
    counter_client.inc_and_mul(5, 2); // This should trigger the "multiple of 5" message
    assert_eq!(40, counter_client.get());

    thread::sleep(Duration::from_millis(1000));
}
