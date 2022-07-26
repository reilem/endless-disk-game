use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast, time::sleep};
use tokio_tungstenite::tungstenite::{
    self,
    protocol::{frame::coding::CloseCode, CloseFrame},
};

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
            log::debug!("Sending ping: {}", count);
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
            let mut websocket = tokio_tungstenite::accept_async(stream)
                .await
                .unwrap_or_else(|err| panic!("Failled to accept websocket: {:?}", err));
            log::info!("Started connection {:?}", addr);
            loop {
                tokio::select! {
                    Some(Ok(msg)) = websocket.next() => {
                        if let Ok(txt) = msg.to_text() {
                            if txt.is_empty() {
                                websocket.close(Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: "Client initiated disconnect".into()
                                }))
                                .await
                                .unwrap_or_else(|err| log::info!("Disconnected {:?} {:?}", addr, err));
                                break;
                            }
                        }
                        log::info!("Received msg {} from {:?}", msg, addr);
                    },
                    Ok(ping) = rx.recv() => {
                        websocket
                            .send(tungstenite::Message::Text(format!("Ping: {}", ping)))
                            .await
                            .unwrap_or_else(|err| log::warn!("Failed to send message: {:?}", err));
                    }
                }
            }
        });
    }
}
