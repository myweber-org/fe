
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

pub fn convert_temperature(value: f64, from_unit: &str, to_unit: &str) -> Option<f64> {
    match (from_unit.to_lowercase().as_str(), to_unit.to_lowercase().as_str()) {
        ("c", "f") => Some(celsius_to_fahrenheit(value)),
        ("f", "c") => Some(fahrenheit_to_celsius(value)),
        ("c", "c") | ("f", "f") => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
        assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    }

    #[test]
    fn test_convert_temperature() {
        assert_eq!(convert_temperature(0.0, "C", "F"), Some(32.0));
        assert_eq!(convert_temperature(100.0, "C", "F"), Some(212.0));
        assert_eq!(convert_temperature(32.0, "F", "C"), Some(0.0));
        assert_eq!(convert_temperature(212.0, "F", "C"), Some(100.0));
        assert_eq!(convert_temperature(25.0, "C", "C"), Some(25.0));
        assert_eq!(convert_temperature(77.0, "F", "F"), Some(77.0));
        assert_eq!(convert_temperature(10.0, "K", "C"), None);
    }
}