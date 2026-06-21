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
    #[serde(default)]
    pub reimburser: String,
    #[serde(default)]
    pub project_code: String,
    #[serde(default)]
    pub page_type: String,
    #[serde(default)]
    pub page_action: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageClassification {
    pub page_number: i32,
    pub original_file: String,
    pub page_type: String,
    pub page_action: String,
    pub confidence: f64,
    pub info: InvoiceInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamingPreview {
    pub template: String,
    pub file_name: String,
    pub warnings: Vec<NamingWarning>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamingWarning {
    pub warning_type: String,
    pub message: String,
    pub field: Option<String>,
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
            commands::classify_pdf_pages,
            commands::batch_split_confirmed,
            commands::preview_file_names,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
