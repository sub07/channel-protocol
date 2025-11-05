use std::{sync::mpsc::Receiver, thread, time::Duration};

use channel_protocol::channel_protocol;

#[channel_protocol]
trait CounterInputProtocol {
    fn get_and_inc(i: i32) -> i32;
    fn inc_and_mul(add: i32, mul: i32) -> i32;
    fn inc(i: i32);
    fn dec(i: i32);
    fn reset();
    fn get() -> i32;
}

#[channel_protocol]
trait CounterOutputProtocol {
    fn reached_10();
    fn multiple_of_5(val: i32);
}

struct CounterApp {
    counter: i32,
    prev_counter: i32,
}

impl HandleCounterInputProtocol<()> for CounterApp {
    fn get_and_inc(&mut self, (): (), i: i32) -> i32 {
        let val = self.counter;
        self.counter += i;
        val
    }

    fn inc_and_mul(&mut self, (): (), add: i32, mul: i32) -> i32 {
        self.counter += add;
        self.counter *= mul;
        self.counter
    }

    fn inc(&mut self, (): (), i: i32) {
        self.counter += i;
    }

    fn dec(&mut self, (): (), i: i32) {
        self.counter -= i;
    }

    fn reset(&mut self, (): ()) {
        self.counter = 0;
    }

    fn get(&mut self, (): ()) -> i32 {
        self.counter
    }
}

impl CounterApp {
    pub const fn new() -> Self {
        Self {
            counter: 0,
            prev_counter: 0,
        }
    }

    pub const fn is_multiple_of_5(&self) -> bool {
        self.counter % 5 == 0
    }

    pub const fn has_reached_10(&self) -> bool {
        self.counter == 10
    }

    pub const fn has_changed(&self) -> bool {
        self.counter != self.prev_counter
    }

    pub const fn save_previous(&mut self) {
        self.prev_counter = self.counter;
    }
}

fn manager_thread(
    counter_outgoing_client: &CounterOutputProtocolClient,
    rx: Receiver<CounterInputProtocolMessage>,
) {
    let mut app = CounterApp::new();
    for message in rx {
        app.save_previous();
        app.dispatch((), message);

        if app.has_changed() {
            if app.has_reached_10() {
                counter_outgoing_client.reached_10();
            }
            if app.is_multiple_of_5() {
                counter_outgoing_client.multiple_of_5(app.counter);
            }
        }
    }
}

fn main() {
    let (counter_client, counter_manager_rx) = CounterInputProtocolClient::new();
    let (counter_outgoing_client, counter_outgoing_rx) = CounterOutputProtocolClient::new();
    thread::spawn(move || {
        manager_thread(&counter_outgoing_client, counter_manager_rx);
    });

    thread::spawn(|| {
        for message in counter_outgoing_rx {
            match message {
                CounterOutputProtocolMessage::Reached10 => {
                    println!("Counter reached 10!");
                }
                CounterOutputProtocolMessage::MultipleOf5(MultipleOf5ParamMessage { val }) => {
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
