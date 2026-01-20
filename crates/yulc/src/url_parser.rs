use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }
        
        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut domain_start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                domain_start = prefix.len();
                break;
            }
        }
        
        let remaining = &url[domain_start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        
        if domain_end == 0 {
            None
        } else {
            Some(remaining[..domain_end].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://subdomain.example.co.uk"),
            Some("subdomain.example.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com"),
            Some("example.com".to_string())
        );
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let mut scheme = String::new();
        let mut remaining = url;

        if let Some(pos) = url.find("://") {
            scheme = url[..pos].to_string();
            remaining = &url[pos + 3..];
        }

        let mut domain_end = remaining.len();
        let mut path_start = domain_end;
        let mut query_start = None;
        let mut fragment_start = None;

        for (i, ch) in remaining.char_indices() {
            match ch {
                '/' if i < path_start => {
                    path_start = i;
                    domain_end = i;
                }
                '?' if query_start.is_none() => {
                    query_start = Some(i);
                    if i < path_start {
                        path_start = i;
                        domain_end = i;
                    }
                }
                '#' if fragment_start.is_none() => {
                    fragment_start = Some(i);
                }
                _ => {}
            }
        }

        let domain = remaining[..domain_end].to_string();
        let path = if path_start < remaining.len() {
            let end = query_start.unwrap_or(fragment_start.unwrap_or(remaining.len()));
            remaining[path_start..end].to_string()
        } else {
            String::new()
        };

        let query_params = if let Some(q_start) = query_start {
            let q_end = fragment_start.unwrap_or(remaining.len());
            Self::parse_query(&remaining[q_start + 1..q_end])
        } else {
            HashMap::new()
        };

        let fragment = fragment_start.map(|f_start| remaining[f_start + 1..].to_string());

        Ok(ParsedUrl {
            scheme,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    fn parse_query(query_str: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for pair in query_str.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                params.insert(key.to_string(), value.to_string());
            }
        }
        params
    }

    pub fn get_domain_root(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let root = parts[parts.len() - 2..].join(".");
            Some(root)
        } else {
            None
        }
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://www.example.com/path/to/resource?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::parse(url).unwrap();

        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "www.example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_domain_root_extraction() {
        let url = "https://subdomain.example.co.uk/page";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.get_domain_root(), Some("example.co.uk".to_string()));
    }

    #[test]
    fn test_url_without_scheme() {
        let url = "example.com/path";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
    }
}