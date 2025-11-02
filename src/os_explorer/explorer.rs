use anyhow::bail;
use std::{fs, path};

pub fn search_files_in_path(
    root_path: &str,
    prefix: &str,
) -> anyhow::Result<(String, Vec<String>)> {
    let mut file_list = vec![];
    for entry in fs::read_dir(root_path)? {
        let path = entry?.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(prefix) {
                    for file_entry in fs::read_dir(&path)? {
                        let file_entry = file_entry?;
                        let file_path = file_entry.path();

                        if file_path.is_file() {
                            if let Some(full_path) = path::absolute(file_path)?.to_str() {
                                file_list.push(full_path.to_string());
                            }
                        }
                    }
                    file_list.sort();
                    return Ok((name.to_string(), file_list));
                }
            }
        }
    }

    bail!("no valid audio folder found that matches the prefix: {prefix}");
}
