use std::{env, fs, process};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};


#[derive(Deserialize)]
struct GitHubRepo {
    stargazers_count: u32,
}

#[derive(Deserialize)]
struct CommitInfo {
    commit: CommitDetails,
}

#[derive(Deserialize)]
struct CommitDetails {
    committer: CommitterInfo,
}

#[derive(Deserialize)]
struct CommitterInfo {
    date: String,
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

fn fetch_github_repo_info(client: &Client, repo: &str, username: &str, token: &str) -> Option<(u32, String)> {
    let api_url = format!("https://api.github.com/repos/{}", repo);
    let stars_response = client.get(&api_url)
        .header("User-Agent", "rust-cli")
        .basic_auth(username, Some(token))
        .send()
        .ok()?;
    
    let stars = if stars_response.status().is_success() {
        let json: GitHubRepo = stars_response.json().ok()?;
        json.stargazers_count
    } else {
        return None;
    };
    
    let commits_url = format!("https://api.github.com/repos/{}/commits?per_page=1", repo);
    let commits_response = client.get(&commits_url)
        .header("User-Agent", "rust-cli")
        .basic_auth(username, Some(token))
        .send()
        .ok()?;
    
    let last_commit_date = if commits_response.status().is_success() {
        let json: Vec<CommitInfo> = commits_response.json().ok()?;
        json.get(0).map(|commit| commit.commit.committer.date.clone()).unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };
    
    Some((stars, last_commit_date))
}

#[allow(non_snake_case)]
fn AI(stars: u32, max_stars: u32) -> bool {
    stars <= max_stars
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
    let three_years_ago = Utc::now() - Duration::days(3 * 365);

    for repo in repos {
        if let Some((stars, last_commit)) = fetch_github_repo_info(&client, &repo, &username, &token) {
            let outdated = if let Ok(date) = DateTime::parse_from_rfc3339(&last_commit) {
                date < three_years_ago
            } else {
                false
            };
            
            let warning_emoji = if outdated { "💀🔥" } else { "✅" };
            let bomb_emoji = if stars < 10 { "💣💣💣" } else if stars < 50 { "💣💣" } else { "💣" };
            
            if AI(stars, max_stars) {
                println!("{} {} has {} stars {} | Last commit: {}", bomb_emoji, repo, stars, warning_emoji, last_commit);
            }
        }
    }
}

