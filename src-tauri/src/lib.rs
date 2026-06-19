use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceInfo {
    pub invoice_number: String,
    pub buyer: String,
    pub seller: String,
    pub amount: String,
    pub date: String,
    pub invoice_type: String,
    pub is_reimbursement: bool,
    pub page_number: i32,
    pub original_file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedInvoice {
    #[serde(flatten)]
    pub info: InvoiceInfo,
    pub output_file_name: String,
    pub output_path: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingProgress {
    pub current_file: String,
    pub current_page: i32,
    pub total_pages: i32,
    pub processed: i32,
    pub total: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub naming_template: String,
    pub output_directory: String,
    pub duplicate_handling: String,
    pub manual_correction: bool,
    pub enable_log: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub mod invoice;
pub mod pdf_utils;
pub mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_invoices,
            commands::get_pdf_info,
            commands::split_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
