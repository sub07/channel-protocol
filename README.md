# Channel Protocol

[<img alt="crates.io" src="https://img.shields.io/crates/v/channel-protocol?style=for-the-badge&logo=rust&logoColor=white">](https://crates.io/crates/channel-protocol)

## What is it ?

A procedural macro to generate channel protocol clients.

You can use function oriented communication between threads instead of communicating by sending messages through channels.

This is an abstraction over channels that makes inter-thread communication easier to use and read.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
channel-protocol = "*"
oneshot = { version = "0.1", features = ["std"], default-features = false } # Used for returned values
```

## Features

- [x] std sync channel
- [ ] async channel (contribution are welcomed)

## Example

Check the [examples](./examples) folder for examples.
