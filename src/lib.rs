mod change;
pub mod changelog;
pub mod config;
pub mod util;
pub mod version;
mod version_file_sync;

use inquire::{
    list_option::ListOption,
    validator::{Validation, ValueRequiredValidator},
    Text,
};
use std::{fs::OpenOptions, io::Write};

use change::commit_changes;
use changelog::generate_and_insert_changelogs;
use colored::Colorize;
use config::Config;
use util::check_for_uncommitted_changes;
use version::{increment_version, write_new_version_to_file, VersionDesignation};
use version_file_sync::sync_version_to_file;

use crate::{
    changelog::TemplateOption,
    version_file_sync::{SupportedSyncFileFormat, VersionSyncFile},
};

pub const EXPECTED_CONFIG_FILE_NAME: &str = "gitscribe.json";

/// Handles the version change when the any subcommand is used
/// # Arguments
/// * `config` - The config struct
/// * `version_designation` - The version designation
/// # Examples
/// ```
/// use gitscribe::Config;
/// let config = Config::create_default();
/// gitscribe::handle_version_bump(config, VersionDesignation::Patch);
/// ```
pub fn handle_version_bump(config: Config, version_designation: VersionDesignation) {
    // check if there are uncommitted changes
    check_for_uncommitted_changes();
    let version = increment_version(&config, &version_designation);
    let config = write_new_version_to_file(config.clone(), version.clone());

    match config.version_sync_files.as_ref() {
        Some(sync_files) => {
            let file_sync_result = sync_version_to_file(sync_files.clone(), version.to_string());
            match file_sync_result {
                Ok(_) => {
                    println!("{}", "All Version Files Updated".green());
                }
                Err(_) => {
                    println!("{}", "Failed to update version in files".red());
                }
            }
        }
        None => {}
    }

    let changelog = generate_and_insert_changelogs(&version, &config);

    match changelog {
        Ok(changelog) => {
            changelog.iter().for_each(|log| println!("{}", log.cyan()));
        }
        Err(_) => {
            println!("{}", "Failed to generate changelog".red());
        }
    }

    commit_changes(version.to_string());
    println!(
        "{} \n {}",
        "✅New version has been committed, and changelog has been updated.".green(),
        "🚀Don't forget to push your changes!".cyan()
    );

    if config.branch_for_release
        && config
            .commands_that_release
            .contains(&version_designation.to_string())
    {
        branch_for_release(&config);
    }
}

/// Handles the initialization of the config file
pub fn handle_init() {
    let config_file = std::fs::read_to_string(EXPECTED_CONFIG_FILE_NAME);
    match config_file {
        Ok(_) => {
            println!(
                "🤬Config file already exists: {}. Please remove it if you want to reinitialize.",
                EXPECTED_CONFIG_FILE_NAME.red()
            );
            std::process::exit(1);
        }
        Err(_) => {}
    }

    let mut config = Config::create_default();
    // walk through the config file creation
    config.version = Text::new("Enter the initial version")
        .with_help_message("The initial version of the application")
        .with_default(config.version.clone().as_str())
        .prompt()
        .unwrap();
    // handle the project repo, defaults to None
    let project_repo = Text::new("Enter the project repository")
        .with_help_message("eg. https://github.com/mikaelmello/inquire/")
        .with_default(
            config
                .project_repo
                .clone()
                .unwrap_or("".to_string())
                .as_str(),
        )
        .prompt()
        .unwrap();
    if project_repo.len() > 0 {
        config.project_repo = Some(project_repo);
    }
    config.branch_for_release = inquire::Confirm::new("Create a branch for releases?")
        .with_help_message("'y' for yes or 'n' for no")
        .prompt()
        .unwrap();
    if config.branch_for_release {
        config.commands_that_release = inquire::MultiSelect::new(
            "Select Commands That Create A Release",
            vec![
                "major".to_string(),
                "minor".to_string(),
                "patch".to_string(),
            ],
        )
        .with_default(&[0, 1]) //default to major and minor
        .with_validator(|input: &[ListOption<&String>]| {
            // ensure major is selected
            if input.iter().any(|option| option.value == "major") {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Major must be selected".into()))
            }
        })
        .prompt()
        .unwrap();
    }

    // handle the changelog output selections
    let mut changelog_output_selections = vec![];
    println!(
        "{}",
        "----------------- Output Files -----------------".green()
    );

    loop {
        let template_option =
            inquire::Select::new("Select A Changelog Template", TemplateOption::values())
                .prompt()
                .unwrap();

        let default_filepath = match template_option {
            TemplateOption::Markdown => "./CHANGELOG.md",
            TemplateOption::VueQuasar => "./src/components/GitscribeChangelog.vue",
        };
        let output_filepath = Text::new("Enter the output filepath")
            .with_help_message("The output filepath relative to the root of the project")
            .with_validator(ValueRequiredValidator::new(
                "The output filepath cannot be empty",
            ))
            .with_default(default_filepath)
            .prompt()
            .unwrap();

        changelog_output_selections.push(config::ChangelogOutputOption {
            template_option,
            output_filepath,
        });
        let add_another = inquire::Confirm::new("Add another changelog file output?")
            .with_help_message("'y' for yes or 'n' for no")
            .prompt()
            .unwrap();
        if !add_another {
            config.changelog_output_selections = changelog_output_selections;
            break;
        }
    }
    let add_version_sync_files = inquire::Confirm::new("Add a file to sync the version with?")
        .with_help_message("'y' for yes or 'n' for no")
        .prompt()
        .unwrap();
    if add_version_sync_files {
        println!(
            "{}",
            "----------------- Files To Sync -----------------".green()
        );
        // handle the version sync files
        let mut version_sync_files = vec![];
        loop {
            let file_path = Text::new("Enter the file path")
                .with_help_message("The file path relative to the root of the project")
                .with_validator(ValueRequiredValidator::new("The file path cannot be empty"))
                .prompt()
                .unwrap();
            let file_format =
                inquire::Select::new("Select A File Format", SupportedSyncFileFormat::values())
                    .prompt()
                    .unwrap();
            let version_key = Text::new("Enter the version key")
                .with_help_message(
                    "The Key in the file to update with the new version. e.g. version",
                )
                .with_default("version")
                .prompt()
                .unwrap();

            version_sync_files.push(VersionSyncFile {
                file_path,
                file_format,
                version_key,
            });
            let add_another = inquire::Confirm::new("Add another version sync file?").with_help_message(
            "If you want to add another version sync file, select yes. Otherwise, select no.",
        )
            .prompt()
            .unwrap();
            if !add_another {
                config.version_sync_files = Some(version_sync_files);
                break;
            }
        }
    }

    // create the file
    let config_file = serde_json::to_string_pretty(&config).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(EXPECTED_CONFIG_FILE_NAME)
        .unwrap();
    file.write_all(config_file.as_bytes()).unwrap();
    println!("{}", "🚀Config file has been initialized.".green());
}

fn branch_for_release(config: &Config) {
    let mut branch_version = config.version.clone();
    // replace the last part of the version with an X
    if !config.commands_that_release.contains(&"patch".to_string()) {
        branch_version = branch_version
            .split(".")
            .take(2)
            .collect::<Vec<&str>>()
            .join(".")
            + ".X";
    }
    // replace the second to last part of the version with an X
    if !config.commands_that_release.contains(&"minor".to_string()) {
        branch_version = branch_version
            .split(".")
            .enumerate()
            .map(|(i, part)| if i == 1 { "X" } else { part })
            .collect::<Vec<&str>>()
            .join(".");
    }

    let output = std::process::Command::new("git")
        .args(&[
            "checkout",
            "-b",
            format!("release/{}", branch_version).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        println!("{}", "✅Branch has been created for the release.".green());
        println!("{}", "  ℹ️ To publish your release run:".green());
        println!(
            "{}",
            format!("      git push origin release/{branch_version}").cyan()
        );
    } else {
        println!(
            "{}",
            format!("🤬Failed to create a branch for the release/{branch_version}.").red()
        );
        std::process::exit(1);
    }
}
