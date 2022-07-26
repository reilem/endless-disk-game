use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast, time::sleep};
use tokio_tungstenite::tungstenite;

#[tokio::main]
async fn main() {
    // Initialise logging
    env_logger::init();

    let (tx, _rx) = broadcast::channel::<u128>(1024);

    let timer_tx = tx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(1000)).await;
        let mut count: u128 = 0;
        log::info!("Starting timer...");
        loop {
            log::info!("Sending ping: {}", count);
            timer_tx
                .send(count)
                .unwrap_or_else(|err| panic!("Failed to send message count: {} {:?}", count, err));
            count += 1;
            sleep(Duration::from_millis(1000)).await;
        }
    });

    log::info!("Setting up tcp listener...");

    let server = TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap_or_else(|err| panic!("Failed to bind tcp listener: {:?}", err));

    while let Ok((stream, addr)) = server.accept().await {
        let mut rx = tx.clone().subscribe();
        tokio::spawn(async move {
            let websocket = tokio_tungstenite::accept_async(stream)
                .await
                .unwrap_or_else(|err| panic!("Failled to accept websocket: {:?}", err));
            let (mut outgoing, mut incoming) = websocket.split();

            loop {
                tokio::select! {
                    Some(Ok(msg)) = incoming.next() => {
                        log::info!("Received msg {} from {:?}", msg, addr);
                    },
                    Ok(ping) = rx.recv() => {
                        outgoing
                            .send(tungstenite::Message::Text(format!("Ping: {}", ping)))
                            .await
                            .unwrap_or_else(|err| log::warn!("Failed to send message: {:?}", err));
                    }
                }
            }
        });
    }
}
