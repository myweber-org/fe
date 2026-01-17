use clap::Parser;
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    owner: String,
    #[arg(short, long)]
    repo: String,
}

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        args.owner, args.repo
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        let open_issues: Vec<&Issue> = issues.iter().filter(|i| i.state == "open").collect();

        println!("Open issues for {}/{}:", args.owner, args.repo);
        for issue in open_issues {
            println!("#{}: {} ({})", issue.number, issue.title, issue.html_url);
        }
        println!("Total open issues: {}", open_issues.len());
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}