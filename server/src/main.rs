use std::{
    thread::{sleep, spawn},
    time::Duration,
};

use bus::{Bus, BusReader};

fn main() {
    // Initialise logging
    env_logger::init();

    log::info!("Setting up timer...");

    let mut bus = Bus::new(10);
    let mut receivers: Vec<BusReader<u128>> = vec![];

    for _ in 0..5 {
        receivers.push(bus.add_rx());
    }

    spawn(move || {
        let mut count: u128 = 0;
        loop {
            log::info!("Sending ping: {}", count);
            bus.broadcast(count);
            count += 1;
            sleep(Duration::from_millis(500));
        }
    });

    let mut threads = vec![];
    for mut rx in receivers {
        let thread = spawn(move || loop {
            if let Ok(ping) = rx.recv() {
                log::info!("Ping: {}", ping);
            }
        });
        threads.push(thread);
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

    for t in threads {
        t.join()
            .unwrap_or_else(|err| panic!("Cannot join thread: {:?}", err));
    }
}
