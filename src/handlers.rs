use printers::printer::{JobStatus, PrinterOption};
use reqwest::Error;
use serde_json::{json, Value};
use std::{io::Write, path::Path};
use tempfile::{Builder, NamedTempFile};
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

    let printer_list = json!(printer_json);

    PrintResult::SUCCESS(printer_list)
}

fn write_data_to_tempfile(file_name: &str, data: &str) -> Result<NamedTempFile, std::io::Error> {
    let mut named_tempfile = Builder::new().suffix(file_name).tempfile()?;

    write!(named_tempfile, "{}", data)?;

    Ok(named_tempfile)
}

async fn contents_from_url(url: &str) -> Result<(String, String), Error> {
    let file_name = Path::new(url)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    println!("Fetching URL: {}", url);

    let text = reqwest::get(url).await?.text().await.unwrap();

    Ok((file_name, text))
}

async fn handle_print(json: Value) -> PrintResult {
    let printer_name: &str = json["system_name"].as_str().unwrap_or("");
    let file_name: String = json["file_name"].as_str().unwrap_or("").to_string();
    let mut data: String = json["data"].as_str().unwrap_or("").to_string();

    if let Some(url) = json["url"].as_str() {
        match contents_from_url(url).await {
            Ok((_url_file_name, url_data)) => {
                //file_name = url_file_name;
                data = url_data;
            }
            Err(_e) => {
                return PrintResult::FAILED(json!({
                    "error": { "message": "Failed to fetch URL", "value": url},
                }))
            }
        }
    }

    if let Some(selected_printer) = printers::get_printer_by_name(printer_name) {
        if let Ok(tempfile) = write_data_to_tempfile(&file_name, &data) {
            let file_path = tempfile.path().to_str().unwrap();
            println!("File path: {}", file_path);
            // if zpl
            //selected_printer.add_option(PrinterOption::new("raw"));

            let job = printers::print_file(
                &selected_printer,
                "/Users/spork/Downloads/Simple Love Vector.jpeg",
            );
            println!("Print job: {:?}", job);

            match job.status {
                JobStatus::SUCCESS => PrintResult::SUCCESS(json!({
                    "message": "Print job submitted successfully",
                    "status": "success",
                    "type": "create_print_job"
                })),
                JobStatus::FAILED => PrintResult::FAILED(json!({
                    "message": "Print job failed",
                    "status": "failure",
                    "type": "create_print_job"
                })),
            }
        } else {
            PrintResult::FAILED(json!({
                "error": { "message": "Failed to write data to temporary file"},
            }))
        }
    } else {
        PrintResult::FAILED(json!({
            "error": { "message": "Invalid printer name provided", "value": printer_name},
        }))
    }
}

async fn handle_client_request(msg: String) -> PrintResult {
    let text = msg;

    let parsed: Value = serde_json::from_str(&text).expect("Error parsing JSON");
    println!("Received message: {:?}", parsed);

    match parsed["type"].as_str() {
        Some("create_print_job") => handle_print(parsed).await,
        Some("list_available_printers") => handle_printer_list(),
        _ => PrintResult::FAILED(json!({
            "error": { "message": "Unsupported message type"},
        })),
    }
}

pub async fn handle_message(msg: Message) -> PrintResult {
    match msg {
        Message::Text(text) => handle_client_request(text).await,
        Message::Binary(data) => {
            let text = String::from_utf8(data).expect("Failed to convert binary data to string");
            handle_client_request(text).await
        }
        Message::Close(_) => handle_disconnect(),
        _ => {
            println!("Unknown message type: {}", msg);
            PrintResult::DISCONNECTED
        }
    }
}
