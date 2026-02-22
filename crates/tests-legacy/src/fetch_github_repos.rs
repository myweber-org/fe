use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    username: String,
}

#[derive(Deserialize)]
struct Repository {
    name: String,
    description: Option<String>,
    stargazers_count: u32,
    html_url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = Client::new();
    let url = format!("https://api.github.com/users/{}/repos", args.username);
    
    let repos: Vec<Repository> = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .send()?
        .json()?;

    println!("Repositories for {}:", args.username);
    for repo in repos {
        println!("- {} ({} stars)", repo.name, repo.stargazers_count);
        if let Some(desc) = repo.description {
            println!("  Description: {}", desc);
        }
        println!("  URL: {}", repo.html_url);
        println!();
    }

    Ok(())
}