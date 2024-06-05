use printers::printer::{JobStatus, PrinterOption};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

#[derive(PartialEq)]
pub enum PrintResult {
    SUCCESS(Value),
    FAILED(Value),
    DISCONNECTED,
}

fn handle_disconnect() -> PrintResult {
    println!("Client disconnected");
    PrintResult::DISCONNECTED
}

fn handle_printer_list() -> PrintResult {
    let printers = printers::get_printers();

    println!("Printers: {:?}", printers);

    let printer_json: Vec<Value> = printers
        .into_iter()
        .map(|printer| json!({"name": printer.name, "system_name": printer.system_name}))
        .collect();

    let printer_list = json!({ "data": printer_json });

    PrintResult::SUCCESS(printer_list)
}

fn handle_print(data: Value) -> PrintResult {
    let printer_name = data["printer_name"]
        .as_str()
        .expect("No printer name provided");

    let contents = data["contents"].as_str().expect("No contents provided");

    let result: PrintResult;

    if let Some(mut selected_printer) = printers::get_printer_by_name(printer_name) {
        selected_printer.add_option(PrinterOption::new("raw"));
        let job = printers::print(&selected_printer, contents.as_bytes());
        println!("Print job: {:?}", job);

        result = match job.status {
            JobStatus::SUCCESS => PrintResult::SUCCESS(json!({
                "data": { "message": "Print job submitted successfully", "status": "success"},
                "type": "print_response"
            })),
            JobStatus::FAILED => PrintResult::FAILED(json!({
                "data": { "message": "Print job failed", "status": "failure"},
                "type": "print_response"
            })),
        };
    } else {
        result = PrintResult::FAILED(json!({
            "error": { "message": "Invalid printer name provided", "value": printer_name},
        }));
    };

    result
}

fn handle_client_request(msg: String) -> PrintResult {
    let text = msg;

    let parsed: Value = serde_json::from_str(&text).expect("Error parsing JSON");
    println!("Received message: {:?}", parsed);

    match parsed["type"].as_str() {
        Some("print") => handle_print(parsed["data"].clone()),
        Some("list") => handle_printer_list(),
        _ => PrintResult::FAILED(json!({
            "error": { "message": "Unsupported message type"},
        })),
    }
}

pub async fn handle_message(msg: Message) -> PrintResult {
    match msg {
        Message::Text(text) => handle_client_request(text),
        Message::Binary(data) => {
            let text = String::from_utf8(data).expect("Failed to convert binary data to string");
            handle_client_request(text)
        }
        Message::Close(_) => handle_disconnect(),
        _ => {
            println!("Unknown message type: {}", msg);
            PrintResult::DISCONNECTED
        }
    }
}
