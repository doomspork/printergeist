    use futures_util::{stream::SplitSink, SinkExt};
    use std::sync::Arc;
    use serde_json::{json, Value};
    use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
    use tokio::{net::TcpStream, sync::Mutex}; 

    fn handle_disconnect() {
        println!("Client disconnected");
    }

    async fn handle_printer_list(outgoing: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>) {
        let printers = printers::get_printers();
        let printer_json: Vec<Value> = printers
            .into_iter()
            .map(|printer| json!({"name": printer.name, "system_name": printer.system_name}))
            .collect();
    
        let printer_list = json!({
            "data": printer_json
        });
    
        let mut outgoing = outgoing.lock().await;
    
        outgoing
            .send(Message::Text(printer_list.to_string()))
            .await
            .expect("Error sending message to WebSocket");
    }

    async fn handle_print(
        data: Value,
        _outgoing: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) {
        let printer_name = data["printer_name"]
            .as_str()
            .expect("No printer name provided");

        let contents = data["contents"]
            .as_str()
            .expect("No contents provided");

        let job = printers::print(printer_name, contents.as_bytes(), Some("Printergeist Job"));

        println!("Print job: {:?}", job);
    }

    async fn handle_client_request(
        msg: String,
        outgoing: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) {
        let text = msg;
    
        let parsed: Value = serde_json::from_str(&text).expect("Error parsing JSON");
        println!("Received message: {:?}", parsed);
    
        match parsed["type"].as_str() {
            Some("print") => {
                handle_print(parsed["data"].clone(), Arc::clone(&outgoing)).await;
            }
            Some("list") => {
                handle_printer_list(Arc::clone(&outgoing)).await;
            }
            _ => {
                println!("Unknown message type");
            }
        }
    }

   pub async fn handle_message(
        msg: Message,
        outgoing: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) -> Option<()>{
        match msg {
            Message::Text(text) => {
                handle_client_request(text, Arc::clone(&outgoing)).await;
                Some(())
            },
            Message::Binary(data) => {
                let text = String::from_utf8(data).expect("Failed to convert binary data to string");
                handle_client_request(text, Arc::clone(&outgoing)).await;
                Some(())
            },
            Message::Close(_)=> {
                handle_disconnect();
                None
            },
            _ => {
                println!("Unknown message type: {}", msg);
                None
            }
        }
    }