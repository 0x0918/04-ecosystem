use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io::copy,
    net::{TcpListener, TcpStream},
    spawn, try_join,
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt, registry, util::SubscriberInitExt, Layer as _,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    upstream_addr: String,
    listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    registry().with(layer).init();
    let config = resolve_config(); // todo
    let config = Arc::new(config);
    info!("Upstream is {}", config.upstream_addr);
    info!("Listening on {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;

    loop {
        let (client, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        let cloned_config = config.clone();
        spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    let client_to_upstream = copy(&mut client_read, &mut upstream_write);
    let upstream_to_client = copy(&mut upstream_read, &mut client_write);
    match try_join!(client_to_upstream, upstream_to_client) {
        Ok((n, m)) => info!(
            "proxy {} bytes from client to upstream, {} bytes from upstream to client",
            n, m
        ),
        Err(e) => warn!("error proxying {:}", e),
    }
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        upstream_addr: "0.0.0.0:8080".to_string(),
        listen_addr: "0.0.0.0:8081".to_string(),
    }
}
