use crate::config::Config;
use crate::EXPECTED_CONFIG_FILE_NAME;
use colored::Colorize;
use std::fs::OpenOptions;
use std::io::Write;

/// Enum representing the different version designations
pub enum VersionDesignation {
    Major,
    Minor,
    Patch,
}
/// Struct representing the version
#[derive(Clone)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}
impl Version {
    /// Creates a new instance of the Version struct from a string
    /// # Arguments
    /// * `version` - The version string ex. "0.0.1"
    pub fn new(version: String) -> Self {
        let version = version.split(".").collect::<Vec<&str>>();
        Version {
            major: version[0].parse().unwrap(), // if unwrap fails, the program will panic w
            minor: version[1].parse().unwrap(),
            patch: version[2].parse().unwrap(),
        }
    }
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Increments the version based on the version designation
/// # Arguments
/// * `config` - The config struct
/// * `version_designation` - The version designation
/// # Returns
/// * The new version
pub fn increment_version(config: &Config, version_designation: VersionDesignation) -> Version {
    let mut version = Version::new(config.version.clone());
    match version_designation {
        VersionDesignation::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        VersionDesignation::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        VersionDesignation::Patch => {
            version.patch += 1;
        }
    }

    println!(
        "{} {}.{}.{}",
        "New version:".underline(),
        version.major.to_string().green(),
        version.minor.to_string().green(),
        version.patch.to_string().green()
    );

    version
}

/// Writes the new version to the config file and returns the new config
/// # Arguments
/// * `config` - The config struct
/// * `version` - The new version
/// # Returns
/// * The new config
pub fn write_new_version_to_file(config: Config, version: Version) -> Config {
    // write new version of config to config file
    let mut new_config = config;
    new_config.version = format!("{}.{}.{}", version.major, version.minor, version.patch);
    let new_config_string = serde_json::to_string_pretty(&new_config).unwrap();
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(EXPECTED_CONFIG_FILE_NAME);

    match file {
        Ok(mut file) => {
            file.write_all(new_config_string.as_bytes())
                .unwrap_or_else(|_| {
                    println!("Failed to write to file");
                });
        }
        Err(_) => {
            println!("Failed to open file");
        }
    }

    new_config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = Version::new("0.0.1".to_string());
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 1);
    }

    #[test]
    fn test_increment_version_patch() {
        let config = Config::create_default();
        let version = increment_version(&config, VersionDesignation::Patch);
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 2);
    }

    #[test]
    fn test_increment_version_minor() {
        let config = Config::create_default();
        let version = increment_version(&config, VersionDesignation::Minor);
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_increment_version_major() {
        let mut config = Config::create_default();
        config.version = "0.1.1".to_string();
        let version = increment_version(&config, VersionDesignation::Major);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }
}
