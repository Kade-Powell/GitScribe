use colored::Colorize;
use regex::Regex;
/// Prints the banner
/// # Examples
/// ```
/// use gitscribe;
/// gitscribe::print_banner();
/// ```
pub fn print_banner() {
    let banner = r#"
    ╔─────────────────────────╗
    │╔═╗┬┌┬┐  ╔═╗┌─┐┬─┐┬┌┐ ┌─┐│
    │║ ╦│ │   ╚═╗│  ├┬┘│├┴┐├┤ │
    │╚═╝┴ ┴   ╚═╝└─┘┴└─┴└─┘└─┘│
    ╚─────────────────────────╝
    "#;
    println!("{}", banner.cyan().bold());
}

/// Checks if there are uncommitted changes
/// if there are, it will print the changes and exit
pub fn check_for_uncommitted_changes() {
    let output = std::process::Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .expect("Failed to execute command");
    if output.stdout.len() > 0 {
        println!(
            "{}",
            "🛑There Are Uncommitted Changes, please commit before trying again:"
                .red()
                .underline()
        );
        let uncommitted_changes = String::from_utf8_lossy(&output.stdout);

        //get list of uncommitted changes
        let uncommitted_changes = uncommitted_changes
            .split("\n")
            .filter(|change| change.len() > 0)
            .map(|change| change.to_string())
            .collect::<Vec<String>>();

        // parsing for pattern https://git-scm.com/docs/git-status#_output
        let re = Regex::new(r"(?<prefix>[M,T,A,R,D,C,U,\?])(?<change>.*$)").unwrap();

        for change in uncommitted_changes {
            let captures = match re.captures(change.as_str()) {
                Some(captures) => captures,
                None => {
                    println!("{}", change);
                    continue;
                }
            };

            println!(
                "{} {}",
                match captures.name("prefix").unwrap().as_str() {
                    "M" => "Modified:".cyan(),
                    "T" => "File Type Changed:".cyan(),
                    "A" => "Added:".green(),
                    "R" => "Renamed:".yellow(),
                    "D" => "Deleted:".red(),
                    "C" => "Copied:".green(),
                    "U" => "Unmerged:".red(),
                    "?" => "Untracked:".purple(),
                    _ => "Unknown:".red(),
                },
                captures.name("change").unwrap().as_str()
            );
        }
        std::process::exit(1);
    }
}
