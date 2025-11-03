// use std::{thread, time::Duration};

// use tokio::spawn;

// pub enum Message {
//     Start { label: String },
//     Stop,
//     GetTotalMessageCount(oneshot::Sender<usize>),
// }

// #[derive(Clone)]
// pub struct MessageSender(tokio::sync::mpsc::Sender<Message>);

// impl MessageSender {
//     pub async fn start<T: Into<String>>(&self, label: T) {
//         self.0
//             .send(Message::Start {
//                 label: label.into(),
//             })
//             .await
//             .unwrap();
//     }

//     pub async fn get_total_message_count(&self) -> usize {
//         let (tx, rx) = oneshot::channel();
//         self.0
//             .send(Message::GetTotalMessageCount(tx))
//             .await
//             .unwrap();
//         rx.await.unwrap()
//     }

//     pub async fn stop(&self) {
//         self.0.send(Message::Stop).await.unwrap();
//     }
// }

// trait MessageProtocol {
//     fn start(label: String);
//     fn stop();
//     fn get_total_message_count() -> usize;
// }

// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);
//     let context = MessageSender(tx);
//     let other_context = context.clone();

//     spawn(async move {
//         loop {
//             dbg!(thread::current().name());
//             context.start("with label").await;
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             context.stop().await;
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             println!("{}", context.get_total_message_count().await);
//         }
//     });

//     spawn(async move {
//         loop {
//             dbg!(thread::current().name());
//             other_context.start("with label 2").await;
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             other_context.stop().await;
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             println!("{}", other_context.get_total_message_count().await);
//         }
//     });

//     let mut i = 0;
//     while let Some(message) = rx.recv().await {
//         match message {
//             Message::Start { label } => println!("start: {label}"),
//             Message::Stop => println!("stop"),
//             Message::GetTotalMessageCount(sender) => {
//                 sender.send(i).unwrap();
//             }
//         }
//         i += 1;
//     }
// }

fn main() {}
