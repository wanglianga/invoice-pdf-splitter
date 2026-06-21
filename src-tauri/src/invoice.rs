use regex::Regex;
use crate::{InvoiceInfo, NamingPreview, NamingWarning, PageClassification};

pub fn extract_invoice_info(text: &str, page_number: i32, original_file: &str) -> InvoiceInfo {
    let invoice_number = extract_invoice_number(text);
    let buyer = extract_buyer(text);
    let seller = extract_seller(text);
    let amount = extract_amount(text);
    let date = extract_date(text);
    let invoice_type = extract_invoice_type(text);
    let is_reimbursement = is_reimbursement_attachment(text);
    let reimburser = extract_reimburser(text);
    let project_code = extract_project_code(text);
    let (page_type, page_action) = classify_page_type(text);

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
        reimburser,
        project_code,
        page_type,
        page_action,
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

fn extract_reimburser(text: &str) -> String {
    let patterns = vec![
        r"报销人[:：]\s*([^\n\r]+)",
        r"申请人[:：]\s*([^\n\r]+)",
        r"经办人[:：]\s*([^\n\r]+)",
        r"填表人[:：]\s*([^\n\r]+)",
        r"报账人[:：]\s*([^\n\r]+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(name) = caps.get(1) {
                    let name = name.as_str().trim();
                    if !name.is_empty() && name.len() >= 2 {
                        return clean_text(name);
                    }
                }
            }
        }
    }

    "未识别".to_string()
}

fn extract_project_code(text: &str) -> String {
    let patterns = vec![
        r"项目编号[:：]\s*([A-Za-z0-9\-_]+)",
        r"项目号[:：]\s*([A-Za-z0-9\-_]+)",
        r"合同编号[:：]\s*([A-Za-z0-9\-_]+)",
        r"合同号[:：]\s*([A-Za-z0-9\-_]+)",
        r"订单号[:：]\s*([A-Za-z0-9\-_]+)",
        r"项目代码[:：]\s*([A-Za-z0-9\-_]+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(code) = caps.get(1) {
                    let code = code.as_str().trim();
                    if !code.is_empty() {
                        return code.to_string();
                    }
                }
            }
        }
    }

    "未识别".to_string()
}

pub fn classify_page_type(text: &str) -> (String, String) {
    let trimmed = text.trim();

    if is_blank_page(trimmed) {
        return ("空白页".to_string(), "skip".to_string());
    }

    if is_itinerary(trimmed) {
        return ("行程单".to_string(), "split".to_string());
    }

    if is_vat_invoice(trimmed) {
        return ("增值税发票".to_string(), "split".to_string());
    }

    if is_reimbursement_cover(trimmed) {
        return ("报销封面".to_string(), "manual".to_string());
    }

    if is_attachment_page(trimmed) {
        return ("附件".to_string(), "merge".to_string());
    }

    if has_invoice_keywords(trimmed) {
        return ("增值税发票".to_string(), "manual".to_string());
    }

    if has_partial_content(trimmed) {
        return ("未知".to_string(), "manual".to_string());
    }

    ("空白页".to_string(), "skip".to_string())
}

fn is_blank_page(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    let printable: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    printable.len() < 10
}

fn is_itinerary(text: &str) -> bool {
    let keywords = vec![
        "行程单",
        "航空运输电子客票",
        "电子客票行程单",
        "机票行程单",
        "铁路电子客票",
        "火车票",
        "客运发票",
        "出租车发票",
        "交通费",
        "登机牌",
        "车票",
        "航班",
        "车次",
        "客票",
        "行程",
    ];
    keywords.iter().any(|k| text.contains(k))
}

fn is_vat_invoice(text: &str) -> bool {
    let strong_keywords = vec![
        "增值税专用发票",
        "增值税普通发票",
        "增值税电子专用发票",
        "增值税电子普通发票",
    ];
    for keyword in &strong_keywords {
        if text.contains(keyword) {
            return true;
        }
    }

    let has_invoice_keyword = text.contains("发票号码")
        || text.contains("发票代码")
        || text.contains("发票号");
    let has_amount = text.contains("价税合计")
        || text.contains("金额")
        || text.contains("合计");
    let has_seller = text.contains("销售方")
        || text.contains("开票方");
    let has_buyer = text.contains("购买方")
        || text.contains("付款方");

    if has_invoice_keyword && has_amount && (has_seller || has_buyer) {
        return true;
    }

    false
}

fn is_reimbursement_cover(text: &str) -> bool {
    let keywords = vec![
        "报销封面",
        "报销单",
        "费用报销单",
        "差旅费报销单",
        "报销审批单",
        "报销申请单",
        "付款申请",
        "费用申请",
    ];

    let cover_indicators = vec![
        "报销人",
        "部门",
        "事由",
        "领导审批",
        "财务审核",
        "主管审批",
    ];

    let keyword_count = keywords.iter().filter(|k| text.contains(*k)).count();
    let indicator_count = cover_indicators.iter().filter(|k| text.contains(*k)).count();

    keyword_count >= 1 || indicator_count >= 2
}

fn is_attachment_page(text: &str) -> bool {
    let keywords = vec![
        "附件",
        "附页",
        "附录",
        "明细表",
        "清单",
        "汇总表",
        "合计表",
        "支付凭证",
        "银行回单",
        "转账凭证",
        "收款凭证",
        "结算单",
        "对账单",
        "确认单",
        "签收单",
        "验收单",
        "合同",
        "协议",
    ];
    keywords.iter().any(|k| text.contains(k))
}

fn has_invoice_keywords(text: &str) -> bool {
    let keywords = vec![
        "发票",
        "税额",
        "税率",
        "价税",
        "开票",
        "校验码",
        "发票代码",
        "发票号码",
    ];
    let count = keywords.iter().filter(|k| text.contains(*k)).count();
    count >= 2
}

fn has_partial_content(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() > 20
}

pub fn classify_pages(texts: &[String], original_file: &str) -> Vec<PageClassification> {
    let mut classifications = Vec::new();

    for (idx, text) in texts.iter().enumerate() {
        let page_number = (idx + 1) as i32;
        let info = extract_invoice_info(text, page_number, original_file);

        let confidence = calculate_confidence(&info, text);

        classifications.push(PageClassification {
            page_number,
            original_file: original_file.to_string(),
            page_type: info.page_type.clone(),
            page_action: info.page_action.clone(),
            confidence,
            info,
        });
    }

    classifications
}

fn calculate_confidence(info: &InvoiceInfo, text: &str) -> f64 {
    let mut score: f64 = 0.0;
    let mut max_score: f64 = 0.0;

    max_score += 1.0;
    if info.invoice_number != "未识别" {
        score += 1.0;
    }

    max_score += 1.0;
    if info.amount != "0.00" {
        score += 1.0;
    }

    max_score += 0.5;
    if info.date != "未知日期" {
        score += 0.5;
    }

    max_score += 0.5;
    if info.seller != "未识别" {
        score += 0.5;
    }

    max_score += 0.5;
    if info.buyer != "未识别" {
        score += 0.5;
    }

    max_score += 0.5;
    if !text.trim().is_empty() {
        score += 0.5;
    }

    if max_score == 0.0 {
        return 0.0;
    }

    (score / max_score).min(1.0)
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
    name = name.replace("{报销人}", &info.reimburser);
    name = name.replace("{项目编号}", &info.project_code);

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

pub fn validate_file_name(template: &str, info: &InvoiceInfo) -> Vec<NamingWarning> {
    let mut warnings = Vec::new();

    let page_number_str = info.page_number.to_string();
    let all_placeholders: Vec<(&str, &str, &str)> = vec![
        ("{发票号码}", &info.invoice_number, "发票号码"),
        ("{购买方}", &info.buyer, "购买方"),
        ("{销售方}", &info.seller, "销售方"),
        ("{金额}", &info.amount, "金额"),
        ("{日期}", &info.date, "日期"),
        ("{票种}", &info.invoice_type, "票种"),
        ("{原页码}", &page_number_str, "原页码"),
        ("{报销人}", &info.reimburser, "报销人"),
        ("{项目编号}", &info.project_code, "项目编号"),
    ];

    for (placeholder, value, field_name) in &all_placeholders {
        if template.contains(placeholder) {
            if value == &"未识别" || value == &"未知日期" || value == &"0.00" || value.is_empty() {
                warnings.push(NamingWarning {
                    warning_type: "missing_field".to_string(),
                    message: format!("模板包含 {} 但该字段未识别，将使用占位文本", field_name),
                    field: Some(field_name.to_string()),
                });
            }
        }
    }

    let preview_name = generate_file_name(template, info);
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for c in preview_name.chars() {
        if invalid_chars.contains(&c) {
            warnings.push(NamingWarning {
                warning_type: "illegal_char".to_string(),
                message: format!("文件名包含非法字符 '{}'，将被替换为下划线", c),
                field: None,
            });
            break;
        }
    }

    let name_without_ext = preview_name.trim_end_matches(".pdf");
    if name_without_ext.len() > 200 {
        warnings.push(NamingWarning {
            warning_type: "path_too_long".to_string(),
            message: format!("文件名长度 {} 超过 200 字符，可能导致路径过长问题", name_without_ext.len()),
            field: None,
        });
    }

    if preview_name.contains("未识别") || preview_name.contains("未知日期") {
        warnings.push(NamingWarning {
            warning_type: "placeholder_in_name".to_string(),
            message: "文件名中包含未识别的占位文本，建议补充对应字段或修改模板".to_string(),
            field: None,
        });
    }

    warnings
}

pub fn preview_naming(template: &str, info: &InvoiceInfo) -> NamingPreview {
    let file_name = generate_file_name(template, info);
    let warnings = validate_file_name(template, info);

    NamingPreview {
        template: template.to_string(),
        file_name,
        warnings,
    }
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

pub fn check_duplicate_names(
    infos: &[InvoiceInfo],
    template: &str,
) -> Vec<NamingWarning> {
    let mut warnings = Vec::new();
    let mut name_map: std::collections::HashMap<String, Vec<i32>> = std::collections::HashMap::new();

    for info in infos {
        if info.page_action == "skip" {
            continue;
        }
        let name = generate_file_name(template, info);
        name_map.entry(name).or_default().push(info.page_number);
    }

    for (name, pages) in &name_map {
        if pages.len() > 1 {
            warnings.push(NamingWarning {
                warning_type: "duplicate_name".to_string(),
                message: format!(
                    "文件名 \"{}\" 重复出现 {} 次（第 {} 页）",
                    name,
                    pages.len(),
                    pages.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("、")
                ),
                field: None,
            });
        }
    }

    warnings
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
            reimburser: "未识别".to_string(),
            project_code: "未识别".to_string(),
            page_type: "增值税发票".to_string(),
            page_action: "split".to_string(),
        };
        let name = generate_file_name("{日期}_{销售方}_{发票号码}_{金额}", &info);
        assert_eq!(name, "20240115_供应商_12345678_100.00.pdf");
    }

    #[test]
    fn test_classify_blank_page() {
        let text = "   \n  \n  ";
        let (page_type, action) = classify_page_type(text);
        assert_eq!(page_type, "空白页");
        assert_eq!(action, "skip");
    }

    #[test]
    fn test_classify_vat_invoice() {
        let text = "增值税专用发票\n发票号码：12345678\n价税合计：¥100.00\n销售方名称：测试公司\n购买方名称：买方公司";
        let (page_type, action) = classify_page_type(text);
        assert_eq!(page_type, "增值税发票");
        assert_eq!(action, "split");
    }

    #[test]
    fn test_classify_itinerary() {
        let text = "航空运输电子客票行程单\n航班号：CA1234\n日期：2024年01月15日";
        let (page_type, action) = classify_page_type(text);
        assert_eq!(page_type, "行程单");
        assert_eq!(action, "split");
    }

    #[test]
    fn test_classify_reimbursement_cover() {
        let text = "差旅费报销单\n报销人：张三\n部门：技术部\n事由：出差";
        let (page_type, action) = classify_page_type(text);
        assert_eq!(page_type, "报销封面");
        assert_eq!(action, "manual");
    }

    #[test]
    fn test_validate_file_name_missing_field() {
        let info = InvoiceInfo {
            invoice_number: "未识别".to_string(),
            buyer: "未识别".to_string(),
            seller: "未识别".to_string(),
            amount: "0.00".to_string(),
            date: "未知日期".to_string(),
            invoice_type: "普通发票".to_string(),
            is_reimbursement: false,
            page_number: 1,
            original_file: "test.pdf".to_string(),
            reimburser: "未识别".to_string(),
            project_code: "未识别".to_string(),
            page_type: "增值税发票".to_string(),
            page_action: "split".to_string(),
        };
        let warnings = validate_file_name("{发票号码}_{日期}_{金额}", &info);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.warning_type == "missing_field"));
    }

    #[test]
    fn test_preview_naming() {
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
            reimburser: "张三".to_string(),
            project_code: "PRJ001".to_string(),
            page_type: "增值税发票".to_string(),
            page_action: "split".to_string(),
        };
        let preview = preview_naming("{日期}_{销售方}_{发票号码}_{金额}", &info);
        assert_eq!(preview.file_name, "20240115_供应商_12345678_100.00.pdf");
        assert!(preview.warnings.is_empty());
    }
}
