use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio::sync::mpsc::{self, Receiver};

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let handle = worker(rx);
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("sending task {}", i);
            tx.send(format!("task {i}")).await.unwrap();
        }
    });
    handle.join().unwrap();
}

fn worker(mut rx: Receiver<String>) -> JoinHandle<()> {
    thread::spawn(move || {
        let (sender, receiver) = std::sync::mpsc::channel();
        while let Some(s) = rx.blocking_recv() {
            let sender_clone = sender.clone();
            thread::spawn(move || {
                let ret = expensive_blocking_task(s);
                sender_clone.send(ret).unwrap();
            });
            let result = receiver.recv().unwrap();
            println!("result: {}", result);
        }
    })
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
