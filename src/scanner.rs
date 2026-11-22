use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct ProjectFiles {
    pub root: PathBuf,
    pub scenes: Vec<PathBuf>,
    pub resources: Vec<PathBuf>,
    pub scripts: Vec<PathBuf>,
    pub shaders: Vec<PathBuf>,
    pub assets: Vec<PathBuf>,
    pub import_files: Vec<PathBuf>,
    pub all_existing_res_paths: HashSet<String>,
    pub all_existing_dir_paths: HashSet<String>,
    pub uid_map: HashMap<String, String>, // uid -> res:// path
    pub scan_errors: Vec<String>,
}

pub fn find_godot_project(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("project.godot").is_file() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn scan_project(root: &Path) -> ProjectFiles {
    let mut files = ProjectFiles {
        root: root.to_path_buf(),
        ..Default::default()
    };

    // Root folder res:// itself is a valid directory
    files.all_existing_dir_paths.insert("res://".to_string());
    files.all_existing_dir_paths.insert("res:///".to_string());

    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        // Skip .godot cache, .git, and target
        if name.starts_with(".godot") || name.starts_with(".git") || name == "target" {
            return false;
        }
        // Skip directories containing .gdignore
        if e.file_type().is_dir() && e.path().join(".gdignore").exists() {
            return false;
        }
        true
    });

    for entry_res in walker {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                files.scan_errors.push(err.to_string());
                continue;
            }
        };

        let path = entry.path().to_path_buf();
        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(e) => {
                files.scan_errors.push(format!("Failed to strip prefix for {:?}: {}", path, e));
                continue;
            }
        };

        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }

        if entry.file_type().is_dir() {
            let res_dir_no_slash = format!("res://{}", rel_str);
            let res_dir_with_slash = format!("res://{}/", rel_str);
            files.all_existing_dir_paths.insert(res_dir_no_slash);
            files.all_existing_dir_paths.insert(res_dir_with_slash);
            continue;
        }

        let res_path = format!("res://{}", rel_str);
        files.all_existing_res_paths.insert(res_path.clone());

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                "tscn" => files.scenes.push(path.clone()),
                "tres" | "material" => files.resources.push(path.clone()),
                "gd" | "cs" => files.scripts.push(path.clone()),
                "gdshader" => files.shaders.push(path.clone()),
                "import" => files.import_files.push(path.clone()),
                "uid" => {
                    // In Godot 4, .uid files contain the UID string for a sibling file
                    if let Ok(uid_content) = fs::read_to_string(&path) {
                        let uid = uid_content.trim().to_string();
                        let target_res = res_path.trim_end_matches(".uid").to_string();
                        files.uid_map.insert(uid, target_res);
                    }
                }
                "png" | "jpg" | "jpeg" | "svg" | "webp" | "ogg" | "wav" | "mp3" | "ttf"
                | "otf" | "woff" | "woff2" | "glb" | "gltf" | "obj" => {
                    files.assets.push(path.clone());
                }
                _ => {}
            }
        }
    }

    files
}
