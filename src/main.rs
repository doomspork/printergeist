pub mod handlers;

use futures_util::{SinkExt, StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::{
    net::{TcpListener, TcpStream},
    time::interval,
};
use tokio_tungstenite::tungstenite::Message;

async fn handle_connection(raw_stream: TcpStream, addr: std::net::SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("WebSocket connection established: {}", addr);

    let (outgoing, mut incoming) = ws_stream.split();

    let outgoing = Arc::new(Mutex::new(outgoing));

    let mut interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            msg = incoming.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if None == handlers::handle_message(msg, outgoing.clone()).await {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
            _ = interval.tick() => {
               outgoing.lock().await.send(Message::Text("ping".to_string())).await.expect("Error sending ping"); 
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let try_socket = TcpListener::bind("127.0.0.1:8080").await;

    let listener = try_socket.expect("Failed to bind");

    while let Ok((raw_stream, addr)) = listener.accept().await {
        println!("Accepted connection from: {}", addr);
        tokio::spawn(handle_connection(raw_stream, addr));
    }
}
