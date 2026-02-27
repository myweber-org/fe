
use std::collections::HashSet;

pub fn clean_data(input: Vec<i32>) -> Vec<i32> {
    let mut seen = HashSet::new();
    input
        .into_iter()
        .filter(|&x| x > 0)
        .filter(|&x| seen.insert(x))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![1, -5, 2, 2, 3, 0, 4, 4, -1];
        let result = clean_data(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }
}