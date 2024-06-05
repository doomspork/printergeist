pub mod handlers;

use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::{
    net::{TcpListener, TcpStream},
    time::interval,
};
use tokio_tungstenite::tungstenite::{Error, Message};

async fn handle_connection(raw_stream: TcpStream) -> Result<(), Error> {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (mut outgoing, mut incoming) = ws_stream.split();

    let mut interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            msg = incoming.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        match handlers::handle_message(msg).await {
                            handlers::PrintResult::DISCONNECTED => {
                                break;
                            }
                            handlers::PrintResult::SUCCESS(json) | handlers::PrintResult::FAILED(json) => {
                                outgoing.send(Message::Text(json.to_string())).await.unwrap_or_else(|_| println!("Error sending message"));
                            }
                        };
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
               outgoing.send(Message::Text("ping".to_string())).await.expect("Error sending ping");
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let try_socket = TcpListener::bind("127.0.0.1:8080").await;

    let listener = try_socket.expect("Failed to bind");

    while let Ok((raw_stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(raw_stream));
    }
}
