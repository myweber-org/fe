use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    let caps = re.captures(url)?;

    let protocol = caps.name("protocol")?.as_str().to_string();
    let domain = caps.name("domain")?.as_str().to_string();
    let path = caps.name("path").map_or("/", |m| m.as_str()).to_string();

    Some(ParsedUrl {
        protocol,
        domain,
        path,
    })
}