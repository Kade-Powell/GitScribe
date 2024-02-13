use indexmap::IndexMap;

use crate::change::get_changes;
use crate::change::Change;
use crate::config::Config;
use crate::version::Version;
use askama::{Error, Template};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "changelog.md.j2")]
struct MarkdownChangelog {
    version: String,
    date: String,
    changes: IndexMap<String, Vec<Change>>,
}

/// The template options
/// # Variants
/// * Markdown - The markdown template
#[derive(Debug, Serialize, Deserialize)]
pub enum TemplateOption {
    Markdown,
}
impl TemplateOption {
    pub fn values() -> Vec<Self> {
        vec![Self::Markdown] // add all your variants here
    }
}
impl std::fmt::Display for TemplateOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TemplateOption::Markdown => write!(f, "Markdown"),
        }
    }
}

/// Generates the changelog
/// # Arguments
/// * `template_option` - The template option
/// * `version` - The new version
/// # Returns
/// * A result containing a success message or an error
pub fn generate_and_insert_changelogs(
    version: &Version,
    config: &Config,
) -> Result<Vec<String>, Error> {
    // for each output selection, generate the changelog
    let changes = get_changes(&config, &version);
    let mut results = vec![];
    config
        .changelog_output_selections
        .iter()
        .for_each(|output_selection| match output_selection.template_option {
            TemplateOption::Markdown => {
                let changelog = MarkdownChangelog {
                    version: version.to_string(),
                    date: Local::now().format("%Y-%m-%d").to_string(),
                    changes: changes.clone(),
                };
                let rendered_log = changelog.render().expect("Failed to render changelog");
                insert_changelog(&output_selection.output_filepath, &rendered_log);
                results.push(format!(
                    " - Generated {} changelog at {}",
                    output_selection.template_option, output_selection.output_filepath
                ));
            }
        });

    Ok(results)
}

/// Inserts the changelog into the changelog file
/// # Arguments
/// * `config` - The config struct
/// * `changelog` - The changelog
pub fn insert_changelog(output_filepath: &str, changelog: &str) {
    std::fs::write(output_filepath, changelog).expect("Failed to write to file");
}
