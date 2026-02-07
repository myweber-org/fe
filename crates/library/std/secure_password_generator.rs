use rand::{thread_rng, Rng};
use std::collections::HashSet;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

#[derive(Debug, Clone)]
pub struct PasswordGenerator {
    length: usize,
    character_sets: Vec<String>,
    exclude_similar: bool,
    exclude_ambiguous: bool,
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            character_sets: vec![
                UPPERCASE.to_string(),
                LOWERCASE.to_string(),
                NUMBERS.to_string(),
                SYMBOLS.to_string(),
            ],
            exclude_similar: true,
            exclude_ambiguous: true,
        }
    }
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(8);
        self
    }

    pub fn exclude_similar(mut self, exclude: bool) -> Self {
        self.exclude_similar = exclude;
        self
    }

    pub fn exclude_ambiguous(mut self, exclude: bool) -> Self {
        self.exclude_ambiguous = exclude;
        self
    }

    pub fn with_character_sets(mut self, sets: Vec<&str>) -> Self {
        self.character_sets = sets.iter().map(|s| s.to_string()).collect();
        self
    }

    fn get_filtered_chars(&self) -> String {
        let mut all_chars: String = self.character_sets.concat();
        
        if self.exclude_similar {
            let similar: HashSet<char> = "il1Lo0O".chars().collect();
            all_chars = all_chars.chars()
                .filter(|c| !similar.contains(c))
                .collect();
        }

        if self.exclude_ambiguous {
            let ambiguous: HashSet<char> = "{}[]()/\\'\"`~,;:.<>".chars().collect();
            all_chars = all_chars.chars()
                .filter(|c| !ambiguous.contains(c))
                .collect();
        }

        all_chars
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length < 8 {
            return Err("Password length must be at least 8 characters");
        }

        if self.character_sets.is_empty() {
            return Err("At least one character set must be specified");
        }

        let available_chars = self.get_filtered_chars();
        if available_chars.is_empty() {
            return Err("No characters available after applying filters");
        }

        let mut rng = thread_rng();
        let mut password_chars: Vec<char> = Vec::with_capacity(self.length);
        
        for set in &self.character_sets {
            if let Some(&ch) = set.chars().collect::<Vec<_>>().choose(&mut rng) {
                password_chars.push(ch);
            }
        }

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..available_chars.len());
            if let Some(ch) = available_chars.chars().nth(idx) {
                password_chars.push(ch);
            }
        }

        rng.shuffle(&mut password_chars);
        
        Ok(password_chars.into_iter().collect())
    }

    pub fn generate_multiple(&self, count: usize) -> Result<Vec<String>, &'static str> {
        let mut passwords = Vec::with_capacity(count);
        for _ in 0..count {
            passwords.push(self.generate()?);
        }
        Ok(passwords)
    }
}

pub fn estimate_strength(password: &str) -> f64 {
    let length = password.len() as f64;
    let unique_chars: HashSet<char> = password.chars().collect();
    let charset_size = unique_chars.len() as f64;
    
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = length.log2() * charset_size.log2();
    
    if has_upper && has_lower {
        score *= 1.2;
    }
    if has_digit {
        score *= 1.1;
    }
    if has_symbol {
        score *= 1.3;
    }
    
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let generator = PasswordGenerator::new();
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), DEFAULT_LENGTH);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new().length(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_multiple_passwords() {
        let generator = PasswordGenerator::new();
        let passwords = generator.generate_multiple(5).unwrap();
        assert_eq!(passwords.len(), 5);
        
        let unique_passwords: HashSet<_> = passwords.iter().collect();
        assert_eq!(unique_passwords.len(), 5);
    }

    #[test]
    fn test_strength_estimation() {
        let weak = "password123";
        let strong = "P@ssw0rd!2024";
        
        let weak_score = estimate_strength(weak);
        let strong_score = estimate_strength(strong);
        
        assert!(strong_score > weak_score);
    }
}