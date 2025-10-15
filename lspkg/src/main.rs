use clap::{Arg, Command as ClapCommand};
use serde_json::Value;
use std::fs;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug)]
struct Package {
    name: String,
    version: String,
    description: String,
    path: String,
    used_by: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let matches = ClapCommand::new("lspkg")
        .version("1.0")
        .author("Your Name")
        .about("Lists all the npm packages in the current directory and its subdirectories.")
        .arg(
            Arg::new("no-header")
                .long("no-header")
                .short('n')
                .help("Do not include header in output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("show-used-by")
                .long("show-used-by")
                .short('s')
                .help("Show the packages that use each package")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("directory")
                .long("directory")
                .short('d')
                .help("Directory to search")
                .value_name("DIR")
                .default_value("."),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Output format")
                .value_name("FORMAT")
                .default_value("default"),
        )
        .arg(
            Arg::new("immediate")
                .long("immediate")
                .short('i')
                .help("Display results immediately")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let include_header = !matches.get_flag("no-header");
    let show_used_by = matches.get_flag("show-used-by");
    let search_directory = matches.get_one::<String>("directory").unwrap();
    let output_format = matches.get_one::<String>("format").unwrap();
    let immediate = matches.get_flag("immediate");

    let packages = parse_all_packages(search_directory, show_used_by)?;

    if immediate {
        for package in &packages {
            println!("{} | {} | {} | {} | {}", package.name, package.version, package.description, package.path, package.used_by.as_deref().unwrap_or(""));
        }
    } else if output_format == "markdown" {
        print_markdown(&packages, include_header, show_used_by);
    } else {
        print_table(&packages, include_header, show_used_by);
    }

    Ok(())
}

fn parse_all_packages(search_directory: &str, show_used_by: bool) -> anyhow::Result<Vec<Package>> {
    let mut packages = Vec::new();
    let files = find_package_json_files(search_directory);

    for file in files {
        if let Some(package) = parse_package_json(&file, show_used_by)? {
            packages.push(package);
        }
    }

    // Sort and unique
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages.dedup_by(|a, b| a.name == b.name && a.path == b.path);

    Ok(packages)
}

fn find_package_json_files(search_directory: &str) -> Vec<String> {
    WalkDir::new(search_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().file_name().map_or(false, |f| f == "package.json"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

fn parse_package_json(path: &str, show_used_by: bool) -> anyhow::Result<Option<Package>> {
    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;

    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if name.is_empty() || name == "null" {
        return Ok(None);
    }

    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("-").to_string();
    let description = json.get("description").and_then(|v| v.as_str()).unwrap_or("-").to_string();

    let used_by = if show_used_by {
        Some(run_depcheck(&name)?)
    } else {
        None
    };

    Ok(Some(Package {
        name,
        version,
        description,
        path: path.to_string(),
        used_by,
    }))
}

fn run_depcheck(package_name: &str) -> anyhow::Result<String> {
    let output = Command::new("depcheck")
        .arg(package_name)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.lines().collect::<Vec<_>>().join(","))
    } else {
        Ok("".to_string())
    }
}

fn print_table(packages: &[Package], include_header: bool, show_used_by: bool) {
    if include_header {
        if show_used_by {
            println!("Name | Version | Description | Path | Is Used By");
        } else {
            println!("Name | Version | Description | Path");
        }
    }

    for package in packages {
        if show_used_by {
            println!("{} | {} | {} | {} | {}", package.name, package.version, package.description, package.path, package.used_by.as_deref().unwrap_or(""));
        } else {
            println!("{} | {} | {} | {}", package.name, package.version, package.description, package.path);
        }
    }
}

fn print_markdown(packages: &[Package], include_header: bool, show_used_by: bool) {
    if include_header {
        if show_used_by {
            println!("| Name | Version | Description | Path | Is Used By |");
            println!("|------|---------|-------------|------|--------------|");
        } else {
            println!("| Name | Version | Description | Path |");
            println!("|------|---------|-------------|------|");
        }
    }

    for package in packages {
        if show_used_by {
            println!("| {} | {} | {} | {} | {} |", package.name, package.version, package.description, package.path, package.used_by.as_deref().unwrap_or(""));
        } else {
            println!("| {} | {} | {} | {} |", package.name, package.version, package.description, package.path);
        }
    }
}
