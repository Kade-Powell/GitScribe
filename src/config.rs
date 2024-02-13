use crate::changelog::TemplateOption;
use crate::version_file_sync::VersionSyncFile;
use crate::EXPECTED_CONFIG_FILE_NAME;
use colored::Colorize;
use serde::{Deserialize, Serialize};

/// Struct Representing the Config file
///
/// # Fields
///
/// * `version` - the application version
/// * `changelog_output_selections` - the list of changelog output selections
/// * `project_repo` - the OPTIONAL project repository - used to make links to commits
/// * `version_sync_files` - the OPTIONAL list of files to sync the version number to. eg. Cargo.toml, package.json, pyproject.toml
///
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub version: String,
    pub changelog_output_selections: Vec<ChangelogOutputOption>,
    pub project_repo: Option<String>,
    pub version_sync_files: Option<Vec<VersionSyncFile>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChangelogOutputOption {
    pub template_option: TemplateOption,
    pub output_filepath: String,
}

impl Config {
    /// Creates a new instance of the Config struct with default values
    pub fn create_default() -> Self {
        Config {
            version: "0.0.1".to_string(),
            project_repo: None,
            changelog_output_selections: vec![ChangelogOutputOption {
                template_option: TemplateOption::Markdown,
                output_filepath: "CHANGELOG.md".to_string(),
            }],
            version_sync_files: None,
        }
    }
}

/// Loads the config file
/// # Returns
/// * An Option containing the Config struct
/// * If the file does not exist, an error message will be printed and None will be returned
pub fn load_config() -> Option<Config> {
    let config_file = std::fs::read_to_string(EXPECTED_CONFIG_FILE_NAME);
    match config_file {
        Ok(config_file) => match serde_json::from_str::<Config>(&config_file) {
            Ok(config) => Some(config),
            Err(msg) => {
                println!("🤬Failed to parse config file: {}", msg.to_string().red());
                None
            }
        },
        Err(_) => {
            println!(
                "🤬Failed to read config file: {}. Please run `gitscribe init` to create a new config file.",
                EXPECTED_CONFIG_FILE_NAME.red()
            );
            None
        }
    }
}
