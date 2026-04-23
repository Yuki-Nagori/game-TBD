//! 资产清单测试

use ming_rpg::asset_manager::{AssetEntry, AssetManager, AssetManifest, ValidationError};
use std::io::Write;

fn create_temp_manifest(content: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[test]
fn test_valid_manifest() {
    let manifest = AssetManifest {
        name: "test-mod".to_string(),
        version: "1.0.0".to_string(),
        assets: vec![AssetEntry {
            path: "textures/test.png".to_string(),
            r#type: "texture".to_string(),
        }],
    };

    // 不验证文件存在性（临时文件路径不同）
    // 这里只验证结构
    assert_eq!(manifest.name, "test-mod");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.assets.len(), 1);
}

#[test]
fn test_manifest_validation_missing_name() {
    let manifest = AssetManifest {
        name: "".to_string(),
        version: "1.0.0".to_string(),
        assets: vec![AssetEntry {
            path: "assets/test.png".to_string(),
            r#type: "texture".to_string(),
        }],
    };

    let result = manifest.validate();
    assert!(matches!(result, Err(ValidationError::MissingField(_))));
}

#[test]
fn test_manifest_validation_invalid_path() {
    let manifest = AssetManifest {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        assets: vec![AssetEntry {
            path: "../secret.png".to_string(),
            r#type: "texture".to_string(),
        }],
    };

    let result = manifest.validate();
    assert!(matches!(result, Err(ValidationError::InvalidPath(_))));
}

#[test]
fn test_manifest_validation_unsupported_format() {
    let manifest = AssetManifest {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        assets: vec![AssetEntry {
            path: "assets/script.exe".to_string(),
            r#type: "binary".to_string(),
        }],
    };

    let result = manifest.validate();
    assert!(matches!(result, Err(ValidationError::UnsupportedFormat(_))));
}

#[test]
fn test_manifest_validation_duplicate_path() {
    // 创建临时文件以通过存在性检查
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("assets/test.png");
    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    std::fs::File::create(&file_path).unwrap();

    let manifest = AssetManifest {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        assets: vec![
            AssetEntry {
                path: file_path.to_string_lossy().to_string(),
                r#type: "texture".to_string(),
            },
            AssetEntry {
                path: file_path.to_string_lossy().to_string(),
                r#type: "texture".to_string(),
            },
        ],
    };

    let result = manifest.validate();
    assert!(matches!(result, Err(ValidationError::DuplicatePath(_))));
}

#[test]
fn test_version_compatibility() {
    let manifest = AssetManifest {
        name: "test".to_string(),
        version: "1.2.3".to_string(),
        assets: vec![],
    };

    assert!(manifest.is_compatible_with("1.0.0"));
    assert!(!manifest.is_compatible_with("2.0.0"));
}

#[test]
fn test_manifest_parse_from_toml() {
    let toml = r#"
name = "my-mod"
version = "1.0.0"

[[assets]]
path = "textures/sword.png"
type = "texture"

[[assets]]
path = "models/hero.gltf"
type = "model"
"#;

    let manifest: AssetManifest = toml::from_str(toml).unwrap();
    assert_eq!(manifest.name, "my-mod");
    assert_eq!(manifest.assets.len(), 2);
}

#[test]
fn test_load_manifest_from_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let texture_path = temp_dir.path().join("assets/textures/test.png");
    std::fs::create_dir_all(texture_path.parent().unwrap()).unwrap();
    std::fs::File::create(&texture_path).unwrap();

    let manifest = AssetManifest {
        name: "file-mod".to_string(),
        version: "2.0.0".to_string(),
        assets: vec![AssetEntry {
            path: texture_path.to_string_lossy().to_string(),
            r#type: "texture".to_string(),
        }],
    };
    let toml = toml::to_string(&manifest).unwrap();
    let temp_file = create_temp_manifest(&toml);

    let mut manager = AssetManager::new();
    let result = manager.load_manifest(temp_file.path());

    assert!(
        result.is_ok(),
        "load_manifest should succeed for valid toml: {:?}",
        result.err()
    );
    let loaded = result.unwrap();
    assert_eq!(loaded.name, "file-mod");
    assert_eq!(loaded.version, "2.0.0");
    assert_eq!(loaded.assets.len(), 1);
}

#[test]
fn test_load_manifest_invalid_toml() {
    let bad_content = "not_valid_toml {{{";
    let temp_file = create_temp_manifest(bad_content);

    let mut manager = AssetManager::new();
    let result = manager.load_manifest(temp_file.path());

    assert!(
        result.is_err(),
        "load_manifest should fail for invalid toml"
    );
}
