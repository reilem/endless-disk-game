use std::{collections::HashMap, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
    time::sleep,
};
use tokio_tungstenite::tungstenite::{
    self,
    protocol::{frame::coding::CloseCode, CloseFrame},
};

#[derive(Debug)]
struct PlayerPosition {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Player {
    position: PlayerPosition,
}

#[derive(Debug)]
struct GameState<'a> {
    players: &'a mut HashMap<u16, Player>,
}

#[derive(Debug)]
struct GameEvent {
    player_id: u16,
    move_direction: String,
}

const UPDATES_PER_SECOND: u8 = 30;

#[tokio::main]
async fn main() {
    // Initialise logging
    env_logger::init();

    // Used to send update pings to clients
    let (downstream_tx, _rx) = broadcast::channel::<String>(1024);
    // Used to send update events to central thread
    let (upsteam_tx, mut upstream_rx) = mpsc::channel::<GameEvent>(512);

    // Central thread maintains active game state in memory
    let timer_tx = downstream_tx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(1000)).await;
        let game_state = GameState {
            players: &mut HashMap::new(),
        };
        let mut count: u128 = 0;
        log::info!("Starting timer...");
        loop {
            tokio::select! {
                _ = sleep(Duration::from_millis((1000.0 / (UPDATES_PER_SECOND as f32)) as u64)) => {
                    log::debug!("Sending ping: {}", count);
                    timer_tx
                        .send(format!("{}={:?}", count, game_state))
                        .unwrap_or_else(|err| panic!("Failed to send message count: {} {:?}", count, err));
                    count += 1;
                }
                Some(GameEvent { player_id, move_direction }) = upstream_rx.recv() => {
                    if let Some(player) = game_state.players.get(&player_id) {
                        let mut x = player.position.x;
                        let mut y = player.position.y;
                        match &*move_direction {
                            "left" => x -= 0.1,
                            "right" => x += 0.1,
                            "down" => y -= 0.1,
                            "up" => y += 0.1,
                            _ => {},
                        }
                        let new_player_position = PlayerPosition { x, y };
                        game_state.players.insert(player_id, Player { position: new_player_position });
                    } else {
                        let new_player_position = PlayerPosition { x: 0.0, y: 0.0 };
                        game_state.players.insert(player_id, Player { position: new_player_position });
                        // game_state.players.insert(player, Player { position: PlayerPosition { x: 0.0, y: 0.0 } });
                    }
                    log::info!("New game state: {:?}", game_state);
                }
            }
        }
    });

    log::info!("Setting up tcp listener...");

    let server = TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap_or_else(|err| panic!("Failed to bind tcp listener: {:?}", err));

    while let Ok((stream, addr)) = server.accept().await {
        let mut rx = downstream_tx.clone().subscribe();
        let tx = upsteam_tx.clone();
        tokio::spawn(async move {
            let mut websocket = tokio_tungstenite::accept_async(stream)
                .await
                .unwrap_or_else(|err| panic!("Failled to accept websocket: {:?}", err));
            log::info!("Started connection {:?}", addr);
            loop {
                tokio::select! {
                    Some(Ok(msg)) = websocket.next() => {
                        if let Ok(txt) = msg.to_text() {
                            let txt_string = txt.to_string();
                            if txt.is_empty() {
                                websocket.close(Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: "Client initiated disconnect".into()
                                }))
                                .await
                                .unwrap_or_else(|err| log::info!("Disconnected {:?} {:?}", addr, err));
                                break;
                            } else {
                                let data: Vec<&str> = txt_string.split('|').collect();
                                if let Ok(player_id) = str::parse::<u16>(data[0]) {
                                    tx.send(GameEvent { player_id, move_direction: data[1].to_string()})
                                    .await
                                    .unwrap_or_else(|err| panic!("Failed to send gae event {:?}", err));
                                }
                            }
                        }
                        log::info!("Received msg {} from {:?}", msg, addr);
                    },
                    Ok(game_state) = rx.recv() => {
                        websocket
                            .send(tungstenite::Message::Text(game_state))
                            .await
                            .unwrap_or_else(|err| log::warn!("Failed to send message: {:?}", err));
                    }
                }
            }
        });
    }
}
