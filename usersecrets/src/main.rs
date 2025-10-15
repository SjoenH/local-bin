use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use roxmltree::Document;
use uuid::Uuid;

fn main() {
    let matches = Command::new("usersecrets")
        .version("1.0")
        .author("Your Name")
        .about("Lists all locations of secrets associated with .csproj files and can create new secret files.")
        .arg(
            Arg::new("create")
                .long("create")
                .help("Create a new secret file and add it to the .csproj file.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("full-path")
                .long("full-path")
                .help("Display the full path of the .csproj files.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("path")
                .help("The path to search for .csproj files")
                .index(1)
                .default_value("."),
        )
        .get_matches();

    let create = matches.get_flag("create");
    let full_path = matches.get_flag("full-path");
    let path = matches.get_one::<String>("path").unwrap();

    if create {
        create_secrets(path);
    } else {
        list_secrets(path, full_path);
    }
}

fn create_secrets(search_path: &str) {
    let secret_id = Uuid::new_v4().to_string();
    let projects = find_csproj_files(search_path);

    for project in projects {
        // Add UserSecretsId to .csproj if not exists
        if let Ok(content) = fs::read_to_string(&project) {
            if !content.contains("<UserSecretsId>") {
                // Find PropertyGroup and add after it
                // This is simplistic; assumes one PropertyGroup
                let new_content = content.replace(
                    "<PropertyGroup>",
                    &format!("<PropertyGroup>\n    <UserSecretsId>{}</UserSecretsId>", secret_id),
                );
                fs::write(&project, new_content).unwrap();
            }
        }

        // Create secrets.json
        let home = dirs::home_dir().unwrap();
        let secrets_dir = home.join(".microsoft").join("usersecrets").join(&secret_id);
        fs::create_dir_all(&secrets_dir).unwrap();
        let secrets_file = secrets_dir.join("secrets.json");
        if !secrets_file.exists() {
            fs::write(secrets_file, "{}").unwrap();
        }
    }
}

fn list_secrets(search_path: &str, full_path: bool) {
    let projects = find_csproj_files(search_path);

    for project in projects {
        if let Ok(content) = fs::read_to_string(&project) {
            let doc = Document::parse(&content).unwrap();
            for node in doc.descendants() {
                if node.tag_name().name() == "UserSecretsId" {
                    let user_secret_id = node.text().unwrap_or("").trim();
                    if !user_secret_id.is_empty() {
                        let home = dirs::home_dir().unwrap();
                        let secrets_file = home.join(".microsoft").join("usersecrets").join(user_secret_id).join("secrets.json");
                        let secrets_path = if secrets_file.exists() {
                            secrets_file.to_string_lossy().to_string()
                        } else {
                            "-".to_string()
                        };

                        let project_name = if full_path {
                            project.to_string_lossy().to_string()
                        } else {
                            project.file_name().unwrap().to_string_lossy().to_string()
                        };

                        println!("{:<120}{}", project_name, secrets_path);
                    }
                }
            }
        }
    }
}

fn find_csproj_files(search_path: &str) -> Vec<PathBuf> {
    WalkDir::new(search_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "csproj"))
        .map(|e| e.path().to_path_buf())
        .collect()
}
