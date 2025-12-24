use clap::{App, Arg};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

fn fetch_issues(owner: &str, repo: &str) -> Result<Vec<Issue>, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
    let client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .send()?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json()?;
        Ok(issues)
    } else {
        Err(format!("Failed to fetch issues: {}", response.status()).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("GitHub Issue Fetcher")
        .version("1.0")
        .author("Your Name")
        .about("Fetches issues from a GitHub repository")
        .arg(
            Arg::with_name("owner")
                .short('o')
                .long("owner")
                .value_name("OWNER")
                .help("Owner of the repository")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("repo")
                .short('r')
                .long("repo")
                .value_name("REPO")
                .help("Repository name")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let owner = matches.value_of("owner").unwrap();
    let repo = matches.value_of("repo").unwrap();

    let issues = fetch_issues(owner, repo)?;

    for issue in issues {
        println!("#{} [{}] {}", issue.number, issue.state, issue.title);
        println!("  {}", issue.html_url);
        println!();
    }

    Ok(())
}