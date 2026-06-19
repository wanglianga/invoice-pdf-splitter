use regex::Regex;
use crate::InvoiceInfo;

pub fn extract_invoice_info(text: &str, page_number: i32, original_file: &str) -> InvoiceInfo {
    let invoice_number = extract_invoice_number(text);
    let buyer = extract_buyer(text);
    let seller = extract_seller(text);
    let amount = extract_amount(text);
    let date = extract_date(text);
    let invoice_type = extract_invoice_type(text);
    let is_reimbursement = is_reimbursement_attachment(text);

    InvoiceInfo {
        invoice_number,
        buyer,
        seller,
        amount,
        date,
        invoice_type,
        is_reimbursement,
        page_number,
        original_file: original_file.to_string(),
    }
}

fn extract_invoice_number(text: &str) -> String {
    let patterns = vec![
        r"发票号码[:：]\s*(\d{8,20})",
        r"发票号[:：]\s*(\d{8,20})",
        r"No\.?\s*(\d{8,20})",
        r"号码[:：]\s*(\d{8,20})",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(num) = caps.get(1) {
                    return num.as_str().to_string();
                }
            }
        }
    }

    "未识别".to_string()
}

fn extract_buyer(text: &str) -> String {
    let patterns = vec![
        r"购买方[\s\S]{0,100}?名\s*称[:：]\s*([^\n\r]+)",
        r"购\s*方[\s\S]{0,100}?名\s*称[:：]\s*([^\n\r]+)",
        r"付款方[\s\S]{0,50}?[:：]\s*([^\n\r]+)",
        r"购买方名称[:：]\s*([^\n\r]+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(name) = caps.get(1) {
                    let name = name.as_str().trim();
                    if !name.is_empty() && name.len() > 2 {
                        return clean_text(name);
                    }
                }
            }
        }
    }

    "未识别".to_string()
}

fn extract_seller(text: &str) -> String {
    let patterns = vec![
        r"销售方[\s\S]{0,100}?名\s*称[:：]\s*([^\n\r]+)",
        r"销\s*方[\s\S]{0,100}?名\s*称[:：]\s*([^\n\r]+)",
        r"收款方[\s\S]{0,50}?[:：]\s*([^\n\r]+)",
        r"销售方名称[:：]\s*([^\n\r]+)",
        r"开票方[\s\S]{0,50}?[:：]\s*([^\n\r]+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(name) = caps.get(1) {
                    let name = name.as_str().trim();
                    if !name.is_empty() && name.len() > 2 {
                        return clean_text(name);
                    }
                }
            }
        }
    }

    "未识别".to_string()
}

fn extract_amount(text: &str) -> String {
    let patterns = vec![
        r"价税合计.*?[¥￥]\s*([0-9,]+\.?\d*)",
        r"合计.*?[¥￥]\s*([0-9,]+\.?\d*)",
        r"小写.*?[¥￥]\s*([0-9,]+\.?\d*)",
        r"金额.*?[¥￥]\s*([0-9,]+\.?\d*)",
        r"[¥￥]\s*([0-9,]+\.\d{2})",
        r"([0-9,]+\.\d{2})\s*元",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(amt) = caps.get(1) {
                    let amt = amt.as_str().replace(",", "");
                    if let Ok(num) = amt.parse::<f64>() {
                        return format!("{:.2}", num);
                    }
                }
            }
        }
    }

    "0.00".to_string()
}

fn extract_date(text: &str) -> String {
    let patterns = vec![
        r"开票日期[:：]\s*(\d{4}[-年/.]\s*\d{1,2}[-月/.]\s*\d{1,2})",
        r"开票日[:：]\s*(\d{4}[-年/.]\s*\d{1,2}[-月/.]\s*\d{1,2})",
        r"日期[:：]\s*(\d{4}[-年/.]\s*\d{1,2}[-月/.]\s*\d{1,2})",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(date) = caps.get(1) {
                    return normalize_date(date.as_str());
                }
            }
        }
    }

    "未知日期".to_string()
}

fn extract_invoice_type(text: &str) -> String {
    let types = vec![
        ("增值税电子专用发票", "增值税电子专用发票"),
        ("增值税电子普通发票", "增值税电子普通发票"),
        ("增值税专用发票", "增值税专用发票"),
        ("增值税普通发票", "增值税普通发票"),
        ("电子专用发票", "电子专用发票"),
        ("电子普通发票", "电子普通发票"),
        ("专用发票", "专用发票"),
        ("普通发票", "普通发票"),
        ("机动车销售统一发票", "机动车销售统一发票"),
        ("二手车销售统一发票", "二手车销售统一发票"),
        ("定额发票", "定额发票"),
        ("卷式发票", "卷式发票"),
        ("电子发票", "电子发票"),
        ("报销单", "报销单"),
        ("费用报销", "费用报销单"),
    ];

    for (keyword, name) in types {
        if text.contains(keyword) {
            return name.to_string();
        }
    }

    "普通发票".to_string()
}

fn is_reimbursement_attachment(text: &str) -> bool {
    let keywords = vec![
        "报销单",
        "费用报销",
        "报销审批",
        "报销申请",
        "报销明细",
        "报销凭证",
        "费用明细",
        "差旅费",
        "招待费",
        "办公费",
    ];

    keywords.iter().any(|k| text.contains(k))
}

fn normalize_date(date: &str) -> String {
    if let Ok(re) = Regex::new(r"(\d{4})[-年/.]\s*(\d{1,2})[-月/.]\s*(\d{1,2})") {
        if let Some(caps) = re.captures(date) {
            let year = caps.get(1).unwrap().as_str();
            let month = caps.get(2).unwrap().as_str();
            let day = caps.get(3).unwrap().as_str();
            let m = month.parse::<i32>().unwrap_or(1);
            let d = day.parse::<i32>().unwrap_or(1);
            return format!("{}{:02}{:02}", year, m, d);
        }
    }
    date.to_string()
}

fn clean_text(text: &str) -> String {
    text.replace("\n", "").replace("\r", "").replace("\t", "").trim().to_string()
}

pub fn generate_file_name(template: &str, info: &InvoiceInfo) -> String {
    let mut name = template.to_string();
    name = name.replace("{发票号码}", &info.invoice_number);
    name = name.replace("{购买方}", &info.buyer);
    name = name.replace("{销售方}", &info.seller);
    name = name.replace("{金额}", &info.amount);
    name = name.replace("{日期}", &info.date);
    name = name.replace("{票种}", &info.invoice_type);
    name = name.replace("{原页码}", &info.page_number.to_string());

    name = sanitize_filename(&name);

    if !name.ends_with(".pdf") {
        name.push_str(".pdf");
    }

    name
}

fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut result = String::new();
    for c in name.chars() {
        if invalid_chars.contains(&c) {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

pub fn resolve_duplicate(
    output_dir: &str,
    file_name: &str,
    mode: &str,
) -> (String, bool) {
    use std::path::Path;

    let file_path = Path::new(output_dir).join(file_name);

    match mode {
        "overwrite" => (file_path.to_string_lossy().to_string(), true),
        "skip" => (file_path.to_string_lossy().to_string(), false),
        "rename" => {
            let mut count = 1;
            let stem = Path::new(file_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("invoice");
            let ext = Path::new(file_name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("pdf");

            let mut new_name = file_name.to_string();
            while file_path.exists() || Path::new(output_dir).join(&new_name).exists() {
                new_name = format!("{}({}).{}", stem, count, ext);
                count += 1;
                if count > 1000 {
                    break;
                }
            }
            (
                Path::new(output_dir)
                    .join(&new_name)
                    .to_string_lossy()
                    .to_string(),
                true,
            )
        }
        _ => (file_path.to_string_lossy().to_string(), true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_invoice_number() {
        let text = "发票号码：12345678";
        assert_eq!(extract_invoice_number(text), "12345678");
    }

    #[test]
    fn test_extract_date() {
        let text = "开票日期：2024年01月15日";
        assert_eq!(extract_date(text), "20240115");
    }

    #[test]
    fn test_generate_file_name() {
        let info = InvoiceInfo {
            invoice_number: "12345678".to_string(),
            buyer: "测试公司".to_string(),
            seller: "供应商".to_string(),
            amount: "100.00".to_string(),
            date: "20240115".to_string(),
            invoice_type: "增值税普通发票".to_string(),
            is_reimbursement: false,
            page_number: 1,
            original_file: "test.pdf".to_string(),
        };
        let name = generate_file_name("{日期}_{销售方}_{发票号码}_{金额}", &info);
        assert_eq!(name, "20240115_供应商_12345678_100.00.pdf");
    }
}
