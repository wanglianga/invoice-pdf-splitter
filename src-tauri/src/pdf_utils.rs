use anyhow::{Context, Result};
use lopdf::{Document, Object, ObjectId};
use std::path::Path;

pub fn get_page_count(pdf_path: &str) -> Result<usize> {
    let doc = Document::load(pdf_path).context("Failed to load PDF document")?;
    Ok(doc.get_pages().len())
}

pub fn extract_text_from_page(pdf_path: &str, page_num: u32) -> Result<String> {
    let doc = Document::load(pdf_path).context("Failed to load PDF document")?;
    let text = doc.extract_text(&[page_num])
        .unwrap_or_else(|_| String::new());
    Ok(text)
}

pub fn extract_text_all_pages(pdf_path: &str) -> Result<Vec<String>> {
    let doc = Document::load(pdf_path).context("Failed to load PDF document")?;
    let pages = doc.get_pages();
    let mut texts = Vec::new();

    for page_num in 1..=pages.len() as u32 {
        let text = doc.extract_text(&[page_num])
            .unwrap_or_else(|_| String::new());
        texts.push(text);
    }

    Ok(texts)
}

pub fn split_pdf_single_page(
    input_path: &str,
    output_path: &str,
    page_num: u32,
) -> Result<()> {
    let mut doc = Document::load(input_path).context("Failed to load PDF document")?;
    let pages = doc.get_pages();

    if page_num == 0 || page_num > pages.len() as u32 {
        anyhow::bail!("Invalid page number: {}", page_num);
    }

    let target_page_id = pages.get(&page_num).copied()
        .context(format!("Page {} not found", page_num))?;

    let page_ids_to_remove: Vec<ObjectId> = pages
        .values()
        .copied()
        .filter(|&id| id != target_page_id)
        .collect();

    for page_id in page_ids_to_remove {
        remove_page_from_tree(&mut doc, page_id)?;
    }

    doc.save(output_path).context("Failed to save PDF document")?;

    Ok(())
}

fn remove_page_from_tree(doc: &mut Document, page_id: ObjectId) -> Result<()> {
    if let Ok(page_obj) = doc.get_object(page_id).cloned() {
        if let Some(parent_ref) = page_obj.as_dict().ok()
            .and_then(|dict| dict.get(b"Parent").ok())
            .and_then(|obj| obj.as_reference().ok())
        {
            if let Ok(parent_obj) = doc.get_object(parent_ref).cloned() {
                if let Ok(parent_dict) = parent_obj.as_dict() {
                    if let Ok(kids_obj) = parent_dict.get(b"Kids") {
                        if let Ok(kids_array) = kids_obj.as_array() {
                            let new_kids: Vec<Object> = kids_array
                                .iter()
                                .filter(|obj| {
                                    if let Ok(ref_id) = obj.as_reference() {
                                        ref_id != page_id
                                    } else {
                                        true
                                    }
                                })
                                .cloned()
                                .collect();

                            if let Ok(parent_dict_mut) = doc.get_object_mut(parent_ref) {
                                if let Some(dict) = parent_dict_mut.as_dict_mut().ok() {
                                    dict.set("Kids", Object::Array(new_kids));
                                }
                            }
                        }
                    }

                    if let Ok(count_obj) = parent_dict.get(b"Count") {
                        if let Ok(count) = count_obj.as_i64() {
                            let new_count = count - 1;
                            if let Ok(parent_dict_mut) = doc.get_object_mut(parent_ref) {
                                if let Some(dict) = parent_dict_mut.as_dict_mut().ok() {
                                    dict.set("Count", Object::Integer(new_count));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn split_pdf_pages(
    input_path: &str,
    output_dir: &str,
    file_prefix: &str,
) -> Result<Vec<String>> {
    let doc = Document::load(input_path).context("Failed to load PDF document")?;
    let pages = doc.get_pages();
    let mut output_files = Vec::new();

    std::fs::create_dir_all(output_dir).ok();

    for (page_num, _) in pages.iter() {
        let output_file = format!("{}/{}{}.pdf", output_dir, file_prefix, page_num);
        let result = split_pdf_single_page(input_path, &output_file, *page_num);
        if result.is_ok() {
            output_files.push(output_file);
        }
    }

    Ok(output_files)
}

pub fn find_pdf_files(path: &str) -> Vec<String> {
    let mut pdf_files = Vec::new();
    let path = Path::new(path);

    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext.to_string_lossy().to_lowercase() == "pdf" {
                pdf_files.push(path.to_string_lossy().to_string());
            }
        }
    } else if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let mut sub_files = find_pdf_files(entry_path.to_string_lossy().as_ref());
                    pdf_files.append(&mut sub_files);
                } else if let Some(ext) = entry_path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "pdf" {
                        pdf_files.push(entry_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    pdf_files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pdf_files() {
        let files = find_pdf_files(".");
        assert!(files.len() >= 0);
    }
}
