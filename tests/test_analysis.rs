use std::fs;

#[path = "../src/analyzer.rs"]
mod analyzer;
#[path = "../src/scanner.rs"]
mod scanner;

#[test]
fn test_scanner_and_analysis_extended() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|_| {
        let p = std::env::temp_dir().join("godot_clean_test_v21");
        let _ = fs::create_dir_all(&p);
        tempfile::TempDir::new_in(&p).unwrap()
    });
    let root = temp_dir.path();

    // 1. Create fake project.godot
    fs::write(root.join("project.godot"), "config_version=5\n").unwrap();

    // 2. Create Godot 4 UID mapping file for player texture
    fs::write(root.join("icon.png"), "image data").unwrap();
    fs::write(root.join("icon.png.uid"), "uid://player123\n").unwrap();

    // 3. Create a bullet scene that will be referenced relatively
    fs::write(root.join("bullet.tscn"), "[gd_scene format=3]\n").unwrap();

    // 4. Create an accessories directory
    let acc_dir = root.join("accessories");
    fs::create_dir_all(&acc_dir).unwrap();

    // 5. Create a folder with .gdignore (should be completely ignored by scanner!)
    let ignored_dir = root.join("dist_python");
    fs::create_dir_all(&ignored_dir).unwrap();
    fs::write(ignored_dir.join(".gdignore"), "").unwrap();
    fs::write(ignored_dir.join("should_be_ignored.png"), "junk").unwrap();

    // 6. Create main scene referencing:
    // - res://missing.png (broken file)
    // - uid://player123 (valid resolved UID)
    // - uid://ghost999 (unresolved UID -> broken)
    // - "bullet.tscn" (valid relative path)
    // - res://accessories/ (valid existing directory path!)
    // - res://nonexistent_folder/ (broken directory path)
    let scene_content = r#"[gd_scene load_steps=5 format=3]
[ext_resource type="Texture2D" path="res://missing.png" id="1"]
[ext_resource type="Texture2D" path="uid://player123" id="2"]
[ext_resource type="Texture2D" path="uid://ghost999" id="3"]
[ext_resource type="PackedScene" path="bullet.tscn" id="4"]
[ext_resource type="Script" path="res://accessories/" id="5"]
[ext_resource type="Script" path="res://nonexistent_folder/" id="6"]
"#;
    fs::write(root.join("main.tscn"), scene_content).unwrap();

    // 7. Create unused asset
    fs::write(root.join("unused_sound.wav"), "sound data").unwrap();

    // 8. Create orphaned .import file (source doesn't exist)
    fs::write(root.join("deleted_asset.png.import"), "import data").unwrap();

    // Run scanner
    let files = scanner::scan_project(root);
    assert_eq!(files.scenes.len(), 2); // main.tscn + bullet.tscn
    assert_eq!(files.assets.len(), 2); // icon.png + unused_sound.wav (should_be_ignored.png is ignored by .gdignore!)
    assert_eq!(files.import_files.len(), 1);
    assert!(files.all_existing_dir_paths.contains("res://accessories/"));

    // Run analyzer
    let result = analyzer::analyze_project(&files);

    // Broken references:
    // 1: res://missing.png
    // 2: uid://ghost999
    // 3: res://nonexistent_folder/
    // (res://accessories/ is valid and NOT broken!)
    assert_eq!(result.broken_references.len(), 3);
    let broken_paths: Vec<_> = result.broken_references.iter().map(|b| b.broken_path.as_str()).collect();
    assert!(broken_paths.contains(&"res://missing.png"));
    assert!(broken_paths.contains(&"uid://ghost999"));
    assert!(broken_paths.contains(&"res://nonexistent_folder/"));
    assert!(!broken_paths.contains(&"res://accessories/"));

    // Orphaned import: deleted_asset.png.import
    assert_eq!(result.orphaned_imports.len(), 1);

    // Unused asset: unused_sound.wav
    assert_eq!(result.unused_assets.len(), 1);
    assert_eq!(result.unused_assets[0], "res://unused_sound.wav");
}
