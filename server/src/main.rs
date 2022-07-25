use std::{thread::sleep, time::Duration};

use tokio::sync::broadcast;

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

    let mut threads = vec![];
    for thread_count in 0..30 {
        let mut rx = tx.clone().subscribe();
        let t = tokio::spawn(async move {
            log::info!("Starting thread: {}", thread_count);
            loop {
                let ping = rx
                    .recv()
                    .await
                    .unwrap_or_else(|err| panic!("Error receiving ping: {:?}", err));
                log::info!("Thread {} received: {}", thread_count, ping);
            }
        });
        threads.push(t);
    }

    for t in threads {
        t.await
            .unwrap_or_else(|err| panic!("Error awaiting thread: {:?}", err));
    }
    // log::info!("Setting up tcp listener...");

    // let server = TcpListener::bind("127.0.0.1:3001")
    //     .unwrap_or_else(|err| panic!("Failed to bind tcp listener: {:?}", err));
    // let mut index = 0;
    // for stream in server.incoming() {
    //     let mut next_rx = receivers[index];
    //     let thread = spawn(move || {
    //         let mut websocket = tungstenite::accept(
    //             stream.unwrap_or_else(|err| panic!("Failed to get stream: {:?}", err)),
    //         )
    //         .unwrap_or_else(|err| panic!("Failled to accept websocket: {:?}", err));

    //         let msg = websocket
    //             .read_message()
    //             .unwrap_or_else(|err| panic!("Failed to read message: {:?}", err));
    //         log::info!("Received message: {:?}, sending response;", msg);
    //         let start_response = tungstenite::Message::Text("Starting Thread".to_owned());
    //         websocket
    //             .write_message(start_response)
    //             .expect("Failed to write message");

    //         for ping in next_rx.iter() {
    //             websocket
    //                 .write_message(tungstenite::Message::Text(format!("Ping: {:?}", ping)))
    //                 .unwrap_or_else(|err| panic!("Failed to write message: {:?}", err));
    //         }
    //     });
    //     threads.push(thread);
    // }
}
