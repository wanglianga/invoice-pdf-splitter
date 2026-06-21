use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::invoice::{
    check_duplicate_names, classify_pages, extract_invoice_info, generate_file_name,
    preview_naming, resolve_duplicate,
};
use crate::pdf_utils::{
    extract_text_all_pages, extract_text_from_page, find_pdf_files, get_page_count,
    split_pdf_single_page,
};
use crate::{AppSettings, InvoiceInfo, NamingPreview, PageClassification, ProcessedInvoice,
    ProcessingProgress};

#[tauri::command]
pub async fn process_invoices(
    app: AppHandle,
    files: Vec<String>,
    settings: AppSettings,
) -> Result<Vec<ProcessedInvoice>, String> {
    let mut all_pdf_files: Vec<String> = Vec::new();

    for file_path in &files {
        let path = Path::new(file_path);
        if path.exists() {
            if path.is_dir() {
                let mut pdfs = find_pdf_files(file_path);
                all_pdf_files.append(&mut pdfs);
            } else {
                all_pdf_files.push(file_path.clone());
            }
        }
    }

    all_pdf_files.sort();
    all_pdf_files.dedup();

    if all_pdf_files.is_empty() {
        return Err("没有找到 PDF 文件".to_string());
    }

    let mut total_pages = 0;
    let mut page_counts = Vec::new();

    for pdf_file in &all_pdf_files {
        match get_page_count(pdf_file) {
            Ok(count) => {
                page_counts.push(count);
                total_pages += count;
            }
            Err(e) => {
                eprintln!("Failed to get page count for {}: {}", pdf_file, e);
                page_counts.push(0);
            }
        }
    }

    let mut results: Vec<ProcessedInvoice> = Vec::new();
    let mut processed = 0;

    std::fs::create_dir_all(&settings.output_directory).map_err(|e| e.to_string())?;

    for (file_idx, pdf_file) in all_pdf_files.iter().enumerate() {
        let file_name = Path::new(pdf_file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.pdf")
            .to_string();

        let page_count = *page_counts.get(file_idx).unwrap_or(&0);

        if page_count == 0 {
            results.push(ProcessedInvoice {
                info: InvoiceInfo {
                    invoice_number: "无法读取".to_string(),
                    buyer: "未知".to_string(),
                    seller: "未知".to_string(),
                    amount: "0.00".to_string(),
                    date: "未知".to_string(),
                    invoice_type: "未知".to_string(),
                    is_reimbursement: false,
                    page_number: 0,
                    original_file: file_name.clone(),
                    reimburser: "未识别".to_string(),
                    project_code: "未识别".to_string(),
                    page_type: "未知".to_string(),
                    page_action: "skip".to_string(),
                },
                output_file_name: "".to_string(),
                output_path: "".to_string(),
                success: false,
                error_message: Some("无法读取 PDF 文件".to_string()),
            });
            continue;
        }

        for page_num in 1..=page_count {
            processed += 1;

            let progress = ProcessingProgress {
                current_file: file_name.clone(),
                current_page: page_num as i32,
                total_pages: page_count as i32,
                processed: processed as i32,
                total: total_pages as i32,
                status: "processing".to_string(),
            };

            let _ = app.emit("processing-progress", &progress);

            let result = process_single_page(pdf_file, &file_name, page_num as u32, &settings);

            match result {
                Ok(invoice) => {
                    let _ = app.emit("invoice-processed", &invoice);
                    results.push(invoice);
                }
                Err(e) => {
                    let failed = ProcessedInvoice {
                        info: InvoiceInfo {
                            invoice_number: "处理失败".to_string(),
                            buyer: "未知".to_string(),
                            seller: "未知".to_string(),
                            amount: "0.00".to_string(),
                            date: "未知".to_string(),
                            invoice_type: "未知".to_string(),
                            is_reimbursement: false,
                            page_number: page_num as i32,
                            original_file: file_name.clone(),
                            reimburser: "未识别".to_string(),
                            project_code: "未识别".to_string(),
                            page_type: "未知".to_string(),
                            page_action: "skip".to_string(),
                        },
                        output_file_name: "".to_string(),
                        output_path: "".to_string(),
                        success: false,
                        error_message: Some(e.to_string()),
                    };
                    let _ = app.emit("invoice-processed", &failed);
                    results.push(failed);
                }
            }
        }
    }

    Ok(results)
}

fn process_single_page(
    pdf_path: &str,
    file_name: &str,
    page_num: u32,
    settings: &AppSettings,
) -> Result<ProcessedInvoice, Box<dyn std::error::Error>> {
    let text = extract_text_from_page(pdf_path, page_num).unwrap_or_else(|_| String::new());

    let info = extract_invoice_info(&text, page_num as i32, file_name);

    let output_file_name = generate_file_name(&settings.naming_template, &info);

    let (output_path, should_write) = resolve_duplicate(
        &settings.output_directory,
        &output_file_name,
        &settings.duplicate_handling,
    );

    if !should_write && settings.duplicate_handling == "skip" {
        return Ok(ProcessedInvoice {
            info,
            output_file_name,
            output_path: output_path.clone(),
            success: true,
            error_message: Some("文件已存在，已跳过".to_string()),
        });
    }

    split_pdf_single_page(pdf_path, &output_path, page_num)?;

    Ok(ProcessedInvoice {
        info,
        output_file_name: Path::new(&output_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&output_file_name)
            .to_string(),
        output_path,
        success: true,
        error_message: None,
    })
}

#[tauri::command]
pub fn get_pdf_info(pdf_path: String) -> Result<(usize, Vec<String>), String> {
    let page_count = get_page_count(&pdf_path).map_err(|e| e.to_string())?;
    let mut texts = Vec::new();

    for page_num in 1..=page_count as u32 {
        let text = extract_text_from_page(&pdf_path, page_num).unwrap_or_else(|_| String::new());
        texts.push(text);
    }

    Ok((page_count, texts))
}

#[tauri::command]
pub fn split_pdf(
    input_path: String,
    output_dir: String,
    naming_template: String,
    duplicate_handling: String,
) -> Result<Vec<String>, String> {
    use crate::invoice::extract_invoice_info as extract_info;

    let page_count = get_page_count(&input_path).map_err(|e| e.to_string())?;
    let mut output_files = Vec::new();

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let file_name = Path::new(&input_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf")
        .to_string();

    for page_num in 1..=page_count as u32 {
        let text = extract_text_from_page(&input_path, page_num).unwrap_or_else(|_| String::new());

        let info = extract_info(&text, page_num as i32, &file_name);
        let output_name = generate_file_name(&naming_template, &info);

        let (output_path, _) = resolve_duplicate(&output_dir, &output_name, &duplicate_handling);

        match split_pdf_single_page(&input_path, &output_path, page_num) {
            Ok(_) => output_files.push(output_path),
            Err(e) => eprintln!("Failed to split page {}: {}", page_num, e),
        }
    }

    Ok(output_files)
}

#[tauri::command]
pub fn classify_pdf_pages(
    pdf_path: String,
) -> Result<Vec<PageClassification>, String> {
    let file_name = Path::new(&pdf_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf")
        .to_string();

    let texts = extract_text_all_pages(&pdf_path).map_err(|e| e.to_string())?;

    let classifications = classify_pages(&texts, &file_name);

    Ok(classifications)
}

#[tauri::command]
pub fn batch_split_confirmed(
    app: AppHandle,
    pdf_path: String,
    output_dir: String,
    naming_template: String,
    duplicate_handling: String,
    confirmed_pages: Vec<PageClassification>,
) -> Result<Vec<ProcessedInvoice>, String> {
    let file_name = Path::new(&pdf_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf")
        .to_string();

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let total = confirmed_pages.len();
    let mut results: Vec<ProcessedInvoice> = Vec::new();

    for (idx, page) in confirmed_pages.iter().enumerate() {
        if page.page_action == "skip" {
            results.push(ProcessedInvoice {
                info: page.info.clone(),
                output_file_name: "".to_string(),
                output_path: "".to_string(),
                success: true,
                error_message: Some("已跳过（空白页/无效页）".to_string()),
            });
            continue;
        }

        if page.page_action == "merge" {
            let output_file_name = generate_file_name(&naming_template, &page.info);
            let (output_path, should_write) =
                resolve_duplicate(&output_dir, &output_file_name, &duplicate_handling);

            if should_write {
                match split_pdf_single_page(&pdf_path, &output_path, page.page_number as u32) {
                    Ok(_) => {
                        results.push(ProcessedInvoice {
                            info: page.info.clone(),
                            output_file_name,
                            output_path,
                            success: true,
                            error_message: None,
                        });
                    }
                    Err(e) => {
                        results.push(ProcessedInvoice {
                            info: page.info.clone(),
                            output_file_name: "".to_string(),
                            output_path: "".to_string(),
                            success: false,
                            error_message: Some(format!("合并附件页拆分失败: {}", e)),
                        });
                    }
                }
            } else {
                results.push(ProcessedInvoice {
                    info: page.info.clone(),
                    output_file_name,
                    output_path: output_path.clone(),
                    success: true,
                    error_message: Some("文件已存在，已跳过".to_string()),
                });
            }
            continue;
        }

        let progress = ProcessingProgress {
            current_file: file_name.clone(),
            current_page: page.page_number,
            total_pages: total as i32,
            processed: (idx + 1) as i32,
            total: total as i32,
            status: "processing".to_string(),
        };
        let _ = app.emit("processing-progress", &progress);

        let text =
            extract_text_from_page(&pdf_path, page.page_number as u32).unwrap_or_else(|_| String::new());

        let mut info = extract_invoice_info(&text, page.page_number, &file_name);
        info.page_type = page.page_type.clone();
        info.page_action = page.page_action.clone();

        let output_file_name = generate_file_name(&naming_template, &info);
        let (output_path, should_write) =
            resolve_duplicate(&output_dir, &output_file_name, &duplicate_handling);

        if !should_write && duplicate_handling == "skip" {
            results.push(ProcessedInvoice {
                info,
                output_file_name,
                output_path: output_path.clone(),
                success: true,
                error_message: Some("文件已存在，已跳过".to_string()),
            });
            continue;
        }

        match split_pdf_single_page(&pdf_path, &output_path, page.page_number as u32) {
            Ok(_) => {
                results.push(ProcessedInvoice {
                    info,
                    output_file_name: Path::new(&output_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&output_file_name)
                        .to_string(),
                    output_path,
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                results.push(ProcessedInvoice {
                    info,
                    output_file_name: "".to_string(),
                    output_path: "".to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn preview_file_names(
    naming_template: String,
    pages: Vec<PageClassification>,
) -> Result<Vec<NamingPreview>, String> {
    let mut previews = Vec::new();

    let infos: Vec<InvoiceInfo> = pages
        .iter()
        .filter(|p| p.page_action != "skip")
        .map(|p| p.info.clone())
        .collect();

    let duplicate_warnings = check_duplicate_names(&infos, &naming_template);

    for page in &pages {
        let preview = preview_naming(&naming_template, &page.info);

        let mut all_warnings = preview.warnings;
        for dw in &duplicate_warnings {
            if dw.message.contains(&format!("第{}页", page.page_number)) {
                all_warnings.push(dw.clone());
            }
        }

        previews.push(NamingPreview {
            template: naming_template.clone(),
            file_name: preview.file_name,
            warnings: all_warnings,
        });
    }

    Ok(previews)
}
