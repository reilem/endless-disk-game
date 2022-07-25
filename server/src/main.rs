use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use std::{thread::sleep, time::Duration};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_tungstenite::tungstenite;

#[tokio::main]
async fn main() {
    // Initialise logging
    env_logger::init();

    let (tx, _rx) = broadcast::channel::<u128>(1024);

    let timer_tx = tx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(1000));
        let mut count: u128 = 0;
        log::info!("Starting timer...");
        loop {
            log::info!("Sending ping: {}", count);
            timer_tx
                .send(count)
                .unwrap_or_else(|err| panic!("Failed to send message count: {} {:?}", count, err));
            count += 1;
            sleep(Duration::from_millis(1000));
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

            if let Some(Ok(msg)) = incoming.next().await {
                if let Ok(text) = msg.to_text() {
                    if text.to_lowercase() != "start" {
                        log::warn!("Received invalid start");
                        return;
                    }
                }
                log::info!("Received start!");
                outgoing
                    .send(tungstenite::Message::Text("Starting updates".to_owned()))
                    .await
                    .unwrap_or_else(|err| log::warn!("Failed to send message: {:?}", err));

                loop {
                    log::info!("{:?} waiting for ping...", addr);
                    // Broadcast is meant for sync stuff and we are using it here concurrently which is not allowed
                    // so his kinda is broken when used async stuff
                    let msg = rx.recv().await.unwrap(); // TODO: Fix this using the rx and tx in tokio_tungstenite examples
                    log::info!("{:?} got ping: {}", addr, msg);
                    outgoing
                        .send(tungstenite::Message::Text(format!("Ping: {}", msg)))
                        .await
                        .unwrap_or_else(|err| log::warn!("Failed to send message: {:?}", err));
                }
            }
        });
    }
}
