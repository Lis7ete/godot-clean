use crate::scanner::ProjectFiles;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct BrokenReference {
    pub source_file: String,
    pub line_number: usize,
    pub broken_path: String,
    pub reason: String,
}

#[derive(Debug, Default, Serialize)]
pub struct AnalysisResult {
    pub broken_references: Vec<BrokenReference>,
    pub orphaned_imports: Vec<String>,
    pub unused_assets: Vec<String>,
    pub read_errors: Vec<String>,
    pub scan_errors: Vec<String>,
    pub total_scenes: usize,
    pub total_scripts: usize,
    pub total_resources: usize,
    pub total_assets: usize,
    pub unused_bytes: u64,
}

struct FileScanOutput {
    referenced_paths: HashSet<String>,
    broken_refs: Vec<BrokenReference>,
    error: Option<String>,
}

pub fn analyze_project(files: &ProjectFiles) -> AnalysisResult {
    let res_regex = Regex::new(r#"res://[a-zA-Z0-9_\-/\.\s]+"#).unwrap();
    let uid_regex = Regex::new(r#"uid://[a-zA-Z0-9]+"#).unwrap();
    let rel_regex = Regex::new(r#"["'](\.{1,2}/[^"'\r\n]+|[a-zA-Z0-9_\-]+\.(?:tscn|tres|gd|cs|png|svg|wav|ogg))["']"#).unwrap();

    let mut files_to_scan = Vec::new();
    let project_godot = files.root.join("project.godot");
    if project_godot.is_file() {
        files_to_scan.push(project_godot);
    }
    files_to_scan.extend(files.scenes.clone());
    files_to_scan.extend(files.resources.clone());
    files_to_scan.extend(files.scripts.clone());
    files_to_scan.extend(files.shaders.clone());

    let scan_outputs: Vec<FileScanOutput> = files_to_scan
        .par_iter()
        .map(|path| {
            scan_single_file(
                path,
                &res_regex,
                &uid_regex,
                &rel_regex,
                files,
            )
        })
        .collect();

    let mut all_referenced: HashSet<String> = HashSet::new();
    let mut broken_references: Vec<BrokenReference> = Vec::new();
    let mut read_errors: Vec<String> = Vec::new();

    for out in scan_outputs {
        all_referenced.extend(out.referenced_paths);
        broken_references.extend(out.broken_refs);
        if let Some(err) = out.error {
            read_errors.push(err);
        }
    }

    // Detect orphaned .import files
    let mut orphaned_imports = Vec::new();
    for import_path in &files.import_files {
        let file_name = import_path.file_name().unwrap().to_string_lossy();
        if file_name.ends_with(".import") {
            let original_name = &file_name[..file_name.len() - 7];
            let source_file = import_path.with_file_name(original_name);
            if !source_file.exists() {
                let rel = import_path
                    .strip_prefix(&files.root)
                    .unwrap_or(import_path)
                    .to_string_lossy()
                    .to_string();
                orphaned_imports.push(rel);
            }
        }
    }

    // Detect unused assets (candidates)
    let mut unused_assets = Vec::new();
    let mut unused_bytes: u64 = 0;

    for asset_path in &files.assets {
        let rel_path = asset_path.strip_prefix(&files.root).unwrap_or(asset_path);
        let res_path = format!("res://{}", rel_path.to_string_lossy().replace('\\', "/"));

        if !all_referenced.contains(&res_path) {
            if let Ok(metadata) = fs::metadata(asset_path) {
                unused_bytes += metadata.len();
            }
            unused_assets.push(res_path);
        }
    }

    AnalysisResult {
        broken_references,
        orphaned_imports,
        unused_assets,
        read_errors,
        scan_errors: files.scan_errors.clone(),
        total_scenes: files.scenes.len(),
        total_scripts: files.scripts.len(),
        total_resources: files.resources.len(),
        total_assets: files.assets.len(),
        unused_bytes,
    }
}

fn scan_single_file(
    file_path: &Path,
    res_regex: &Regex,
    uid_regex: &Regex,
    rel_regex: &Regex,
    files: &ProjectFiles,
) -> FileScanOutput {
    let mut referenced = HashSet::new();
    let mut broken = Vec::new();

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return FileScanOutput {
                referenced_paths: referenced,
                broken_refs: broken,
                error: Some(format!("Failed to read {}: {}", file_path.display(), e)),
            };
        }
    };

    let rel_source = file_path
        .strip_prefix(&files.root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let source_dir = file_path.parent().unwrap_or(&files.root);

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // 1. Check res:// links
        for mat in res_regex.find_iter(line) {
            let raw = mat.as_str().trim_end_matches(['"', '\'', ')', '>', ',', ';']);
            let clean = if let Some(idx) = raw.find("::") {
                &raw[..idx]
            } else {
                raw
            };

            // If it's a directory reference (ends with / or matches a known directory)
            let is_dir = clean.ends_with('/') || files.all_existing_dir_paths.contains(clean);
            if is_dir {
                let clean_with_slash = format!("{}/", clean.trim_end_matches('/'));
                let clean_no_slash = clean.trim_end_matches('/');
                if !files.all_existing_dir_paths.contains(&clean_with_slash)
                    && !files.all_existing_dir_paths.contains(clean_no_slash)
                {
                    broken.push(BrokenReference {
                        source_file: rel_source.clone(),
                        line_number: line_num,
                        broken_path: clean.to_string(),
                        reason: "Directory does not exist in project".to_string(),
                    });
                }
                continue;
            }

            referenced.insert(clean.to_string());
            if !files.all_existing_res_paths.contains(clean) {
                broken.push(BrokenReference {
                    source_file: rel_source.clone(),
                    line_number: line_num,
                    broken_path: clean.to_string(),
                    reason: "File does not exist in project".to_string(),
                });
            }
        }

        // 2. Check uid:// links
        for mat in uid_regex.find_iter(line) {
            let uid = mat.as_str().trim_end_matches(['"', '\'', ')', '>', ',', ';']);
            if let Some(target_res) = files.uid_map.get(uid) {
                referenced.insert(target_res.clone());
            } else {
                if !line.contains(&format!("uid=\"{}\"", uid)) {
                    broken.push(BrokenReference {
                        source_file: rel_source.clone(),
                        line_number: line_num,
                        broken_path: uid.to_string(),
                        reason: "Unresolved Godot 4 UID".to_string(),
                    });
                }
            }
        }

        // 3. Check relative file paths, e.g. "bullet.tscn" or "./enemy.gd"
        for cap in rel_regex.captures_iter(line) {
            if let Some(matched_rel) = cap.get(1) {
                let rel_str = matched_rel.as_str();
                let candidate_path = source_dir.join(rel_str);
                if let Ok(canonical) = candidate_path.canonicalize() {
                    if let Ok(from_root) = canonical.strip_prefix(&files.root) {
                        let res_p = format!("res://{}", from_root.to_string_lossy().replace('\\', "/"));
                        referenced.insert(res_p);
                    }
                }
            }
        }
    }

    FileScanOutput {
        referenced_paths: referenced,
        broken_refs: broken,
        error: None,
    }
}
