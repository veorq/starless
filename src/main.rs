use std::{env, fs, process};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct GitHubRepo {
    stargazers_count: u32,
}

fn extract_github_repos(go_mod_content: &str) -> Vec<String> {
    let mut repos = Vec::new();
    let re = Regex::new(r"github\.com/([a-zA-Z0-9_.-]+)/([a-zA-Z0-9_.-]+)").unwrap();
    for cap in re.captures_iter(go_mod_content) {
        if let (Some(org), Some(repo)) = (cap.get(1), cap.get(2)) {
            repos.push(format!("{}/{}", org.as_str(), repo.as_str()));
        }
    }
    repos
}

fn load_github_credentials(config_path: &str) -> Option<(String, String)> {
    let config_content = fs::read_to_string(config_path).ok()?;
    let config: HashMap<String, String> = serde_json::from_str(&config_content).ok()?;
    let username = config.get("username")?.to_string();
    let token = config.get("token")?.to_string();
    Some((username, token))
}

fn fetch_github_stars(client: &Client, repo: &str, username: &str, token: &str) -> Option<u32> {
    let api_url = format!("https://api.github.com/repos/{}", repo);
    let response = client.get(&api_url)
        .header("User-Agent", "rust-cli")
        .basic_auth(username, Some(token))
        .send()
        .ok()?;
    
    if response.status().is_success() {
        let json: GitHubRepo = response.json().ok()?;
        Some(json.stargazers_count)
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <go.mod file> <max stars> <config file>", args[0]);
        process::exit(1);
    }

    let go_mod_path = &args[1];
    let max_stars: u32 = args[2].parse().expect("Invalid number for max stars");
    let config_path = &args[3];

    let (username, token) = load_github_credentials(config_path)
        .expect("Failed to load GitHub credentials from config file");

    let go_mod_content = fs::read_to_string(go_mod_path)
        .expect("Failed to read go.mod file");
    
    let repos = extract_github_repos(&go_mod_content);
    let client = Client::new();

    for repo in repos {
        if let Some(stars) = fetch_github_stars(&client, &repo, &username, &token) {
            if stars <= max_stars {
                println!("{} has {} stars", repo, stars);
            }
        }
    }
}

