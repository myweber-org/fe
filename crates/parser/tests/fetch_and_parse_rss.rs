use clap::Parser;
use reqwest;
use rss::Channel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let content = reqwest::get(&args.url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("Feed Description: {}", channel.description());
    println!("\n--- Items ---\n");

    for item in channel.items() {
        println!("Title: {}", item.title().unwrap_or("No title"));
        println!("Link: {}", item.link().unwrap_or("No link"));
        println!("Description: {}", item.description().unwrap_or("No description"));
        println!();
    }

    Ok(())
}