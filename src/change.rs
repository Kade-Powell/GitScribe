use indexmap::IndexMap;

use crate::{config::Config, version::Version};

use chrono::{Local, NaiveDateTime};
use regex::Regex;

pub const VERSION_COMMIT_MESSAGE: &str = "chore: 📝 update changelog and bump version to ";
//function to add the generated changelog, and updated version to a git commit and commit it
/// # Arguments
/// * `version` - The new version
/// # Panics
/// * If the git command fails
/// * If there are uncommitted changes
pub fn commit_changes(version: String) {
    // add the changes to the git commit
    let output = std::process::Command::new("git")
        .args(&["add", "."])
        .output()
        .expect("Failed to execute command");

    let output = String::from_utf8_lossy(&output.stdout);
    println!("{}", output);

    let output = std::process::Command::new("git")
        .args(&[
            "commit",
            "-m",
            format!("{VERSION_COMMIT_MESSAGE}{version}").as_str(),
        ])
        .output()
        .expect("Failed to execute command");

    let output = String::from_utf8_lossy(&output.stdout);
    println!("{}", output);
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub enum ChangeType {
    Feature,
    Fix,
    Version,
    Unknown,
}
impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChangeType::Feature => write!(f, "Feature"),
            ChangeType::Fix => write!(f, "Fix"),
            ChangeType::Version => write!(f, "Version"),
            ChangeType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub struct Change {
    pub message: String,
    pub commit_id: String,
    pub link: Option<String>,
    pub author: String,
    pub change_type: ChangeType,
    pub date: NaiveDateTime,
}

/// Parses the change from the git log
/// # Arguments
/// * `change` - The change string
/// # Returns
/// * A Change struct
fn parse_change(change: &str, config: &Config) -> Change {
    let commit_id_regex = Regex::new(r"COMMIT_ID:(.*?)AUTHOR:").unwrap();
    let author_regex = Regex::new(r"AUTHOR:(.*?)MESSAGE:").unwrap();
    let message_regex = Regex::new(r"MESSAGE:(.*?)DATE:").unwrap();
    let date_regex = Regex::new(r"DATE:(.*?)--date=iso-strict").unwrap();

    let commit_id = commit_id_regex
        .captures(change)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .to_string();
    let author = author_regex
        .captures(change)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .to_string();
    let message = message_regex
        .captures(change)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .replace(":sparkles:", "✨")
        .replace(":bug:", "🐛") //TODO: cover more gitmoji
        .trim()
        .to_string();
    let link = match &config.project_repo {
        Some(repo) => {
            //strip ending "/" if it exists
            let repo = repo.trim_end_matches('/');

            //handle different git providers
            if repo.contains("github") {
                Some(format!(
                    "{repo}/commit/{commit_id}",
                    repo = repo,
                    commit_id = commit_id
                ))
            } else if repo.contains("stash/projects") {
                // bitbucket
                Some(format!(
                    "{repo}/commits/{commit_id}",
                    repo = repo,
                    commit_id = commit_id
                ))
            } else {
                None
            }
        }
        None => None,
    };

    //parse git log, if the change was released prior to the current version, we will add the previous version to the change
    //TODO: need to parse in a strict way
    let change_type = if message.contains("feat:") {
        ChangeType::Feature
    } else if message.contains("fix:") {
        ChangeType::Fix
    } else if message.contains(VERSION_COMMIT_MESSAGE) {
        ChangeType::Version
    } else {
        ChangeType::Unknown
    };
    let change_date = date_regex
        .captures(change)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .trim()
        .to_string();

    Change {
        message,
        commit_id,
        link,
        author,
        change_type,
        date: NaiveDateTime::parse_from_str(&change_date, "%a %b %d %T %Y %z").unwrap(),
    }
}

/// Gets the changes from the git log
/// # Returns
/// * A vector of Change structs
pub fn get_changes(config: &Config, version: &Version) -> IndexMap<String, Vec<Change>> {
    let output = std::process::Command::new("git")
        .args(&[
            "log",
            "--pretty=format:COMMIT_ID:%H AUTHOR:%an MESSAGE:%s DATE:%cd --date=iso-strict",
        ])
        .output()
        .expect("Failed to execute command");

    let output = String::from_utf8_lossy(&output.stdout);

    let mut changes: Vec<Change> = output
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .filter(|change| change.len() > 0)
        .map(|change| parse_change(change, config))
        .filter(|change| change.change_type != ChangeType::Unknown)
        .collect();

    let mut version_changes = changes
        .clone()
        .into_iter()
        .filter(|change| change.change_type == ChangeType::Version)
        .collect::<Vec<Change>>();
    // add one version change to the version_changes to account for the not yet committed version change
    version_changes.push(Change {
        message: format!("{VERSION_COMMIT_MESSAGE}{version}"),
        commit_id: "HEAD".to_string(),
        change_type: ChangeType::Version,
        author: "GitScribe".to_string(),
        date: Local::now().naive_local(),
        link: None,
    });
    // reverse sorting to get latest changes first
    version_changes.sort_by(|a, b| b.date.cmp(&a.date));

    //filter changes to exclude version changes
    changes = changes
        .into_iter()
        .filter(|change| change.change_type != ChangeType::Version)
        .collect();

    // loop through changes, sort by change date. we want to associate change date for the version change
    let mut change_map: IndexMap<String, Vec<Change>> = IndexMap::new();
    //insert version change as the key, it is sorted above by date so we want to keep the order
    for version_change in &version_changes {
        change_map.insert(
            parse_change_for_version(&version_change.message),
            Vec::new(),
        );
    }

    for change in &mut changes {
        let release_change = version_changes
            .iter()
            .filter(|version_change| version_change.date >= change.date)
            .min_by(|a, b| a.date.cmp(&b.date));

        let version = parse_change_for_version(&release_change.unwrap().message);
        match release_change {
            Some(_) => {
                if change_map.contains_key(&version) {
                    change_map.get_mut(&version).unwrap().push(change.clone());
                } else {
                    println!("Change Map Did not include the version.");
                    std::process::exit(1);
                }
            }
            None => {}
        }
    }
    change_map
}

fn parse_change_for_version(message: &String) -> String {
    let version_regex = Regex::new(r"^.*(\d+\.\d+\.\d+)$").unwrap();
    version_regex
        .captures(message)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use chrono::NaiveDateTime;

    #[test]
    fn test_parse_change() {
        let config = Config::create_default();
        let change = parse_change(
            "COMMIT_ID:123abc AUTHOR:John Doe MESSAGE:feat: :sparkles: add new feature DATE:2024-02-10T00:40:40-05:00 --date=iso-strict",
            &config,
        );
        assert_eq!(change.message, "✨ add new feature");
        assert_eq!(change.commit_id, "123");
        assert_eq!(change.author, "John Doe");
        assert_eq!(change.link, None);
        assert_eq!(change.change_type, ChangeType::Feature);
        assert_eq!(
            change.date,
            NaiveDateTime::parse_from_str("2024-02-09", "%Y-%m-%d").unwrap()
        );
    }

    #[test]
    fn test_get_changes() {
        let config = Config::create_default();
        let changes = get_changes(&config, &Version::new("1.0.0".to_string()));
        assert_eq!(changes.len() > 0, true);
    }
}
