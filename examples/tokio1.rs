use std::{
    thread::{self},
    time::Duration,
};

use tokio::{
    fs::read,
    runtime::{Builder, Runtime},
    task::spawn_blocking,
    time::sleep,
};

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(run(&rt));
    });
    handle.join().unwrap();
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}

async fn run(rt: &Runtime) {
    rt.spawn(async {
        println!("future 1");
        let content = read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });
    rt.spawn(async {
        println!("future 2");
        let result = spawn_blocking(|| expensive_blocking_task("hello".to_string()))
            .await
            .unwrap();
        println!("result: {}", result);
    });
    sleep(Duration::from_secs(1)).await;
}
