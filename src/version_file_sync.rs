use std::io::Error;

use serde::{Deserialize, Serialize};

/// Enum Representing the supported file formats for the version sync file
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SupportedSyncFileFormat {
    Json,
    CargoToml,
    PoetryToml,
    Yaml,
}
impl SupportedSyncFileFormat {
    pub fn values() -> Vec<Self> {
        vec![Self::Json, Self::CargoToml, Self::PoetryToml, Self::Yaml]
    }
}
impl std::fmt::Display for SupportedSyncFileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SupportedSyncFileFormat::Json => write!(f, "Json"),
            SupportedSyncFileFormat::CargoToml => write!(f, "CargoToml"),
            SupportedSyncFileFormat::PoetryToml => write!(f, "PoetryToml"),
            SupportedSyncFileFormat::Yaml => write!(f, "Yaml"),
        }
    }
}

/// Struct Representing the file which other package managers use to store the version number that needs to be updated by gitscribe
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionSyncFile {
    pub file_format: SupportedSyncFileFormat,
    pub file_path: String,
    pub version_key: String,
}

/// Function to sync the version number to the file which other package managers use to store the version number
/// # Arguments
/// * `version_files` - A vector of VersionSyncFile which contains the file format, file path and the version key
/// * `version` - The version number to be updated in the file
/// # Returns
/// * `Result<(), Error>` - A Result of unit type and Error
pub fn sync_version_to_file(
    version_files: Vec<VersionSyncFile>,
    version: String,
) -> Result<(), Error> {
    for version_file in version_files {
        match version_file.file_format {
            SupportedSyncFileFormat::Json => {
                let file = std::fs::read_to_string(&version_file.file_path).unwrap();
                let mut json: serde_json::Value = serde_json::from_str(&file).unwrap(); // Declare json as mutable
                let json_obj = json.as_object_mut().unwrap(); // Get a mutable reference to the json object
                json_obj.insert(
                    version_file.version_key,
                    serde_json::Value::String(version.clone()),
                );
                let json = serde_json::to_string_pretty(&json).unwrap();
                std::fs::write(&version_file.file_path, json).unwrap();
            }
            SupportedSyncFileFormat::CargoToml => {
                let file = std::fs::read_to_string(&version_file.file_path).unwrap();
                let mut toml = toml::from_str::<toml::Value>(&file).unwrap();
                if let Some(package) = toml.as_table_mut().unwrap().get_mut("package") {
                    package.as_table_mut().unwrap().insert(
                        version_file.version_key,
                        toml::Value::String(version.clone()),
                    );
                }
                let toml = toml::to_string(&toml).unwrap();
                std::fs::write(&version_file.file_path, toml).unwrap();
            }
            SupportedSyncFileFormat::PoetryToml => {
                let file = std::fs::read_to_string(&version_file.file_path).unwrap();
                let mut toml = toml::from_str::<toml::Value>(&file).unwrap();
                if let Some(tool) = toml.as_table_mut().unwrap().get_mut("tool") {
                    if let Some(poetry) = tool.as_table_mut().unwrap().get_mut("poetry") {
                        poetry.as_table_mut().unwrap().insert(
                            version_file.version_key,
                            toml::Value::String(version.clone()),
                        );
                    }
                }
                let toml = toml::to_string(&toml).unwrap();
                std::fs::write(&version_file.file_path, toml).unwrap();
            }
            SupportedSyncFileFormat::Yaml => {
                let file = std::fs::read_to_string(&version_file.file_path).unwrap();
                let mut yaml = serde_yaml::from_str::<serde_yaml::Value>(&file).unwrap();
                yaml.as_mapping_mut().unwrap().insert(
                    serde_yaml::Value::String(version_file.version_key),
                    serde_yaml::Value::String(version.clone()),
                );
                let yaml = serde_yaml::to_string(&yaml).unwrap();
                std::fs::write(&version_file.file_path, yaml).unwrap();
            }
        }

        println!("✅Updated version in {}", version_file.file_path.as_str());
    }

    Ok(())
}
