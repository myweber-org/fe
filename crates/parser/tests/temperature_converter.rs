use std::io;

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

struct Temperature {
    value: f64,
    unit: TemperatureUnit,
}

impl Temperature {
    fn new(value: f64, unit: TemperatureUnit) -> Self {
        Temperature { value, unit }
    }

    fn to_celsius(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => self.value,
            TemperatureUnit::Fahrenheit => (self.value - 32.0) * 5.0 / 9.0,
        }
    }

    fn to_fahrenheit(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => (self.value * 9.0 / 5.0) + 32.0,
            TemperatureUnit::Fahrenheit => self.value,
        }
    }

    fn display(&self) -> String {
        let unit_str = match self.unit {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        };
        format!("{:.1}{}", self.value, unit_str)
    }
}

fn main() {
    println!("Temperature Converter");
    println!("=====================");

    loop {
        println!("\nSelect conversion:");
        println!("1. Celsius to Fahrenheit");
        println!("2. Fahrenheit to Celsius");
        println!("3. Exit");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        match choice.trim() {
            "1" => convert_temperature(TemperatureUnit::Celsius, TemperatureUnit::Fahrenheit),
            "2" => convert_temperature(TemperatureUnit::Fahrenheit, TemperatureUnit::Celsius),
            "3" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid choice. Please enter 1, 2, or 3."),
        }
    }
}

fn convert_temperature(from: TemperatureUnit, to: TemperatureUnit) {
    let from_str = match from {
        TemperatureUnit::Celsius => "Celsius",
        TemperatureUnit::Fahrenheit => "Fahrenheit",
    };
    let to_str = match to {
        TemperatureUnit::Celsius => "Celsius",
        TemperatureUnit::Fahrenheit => "Fahrenheit",
    };

    println!("Enter temperature in {}:", from_str);
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let value: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid number. Please enter a valid temperature.");
            return;
        }
    };

    let temp = Temperature::new(value, from);
    let converted_value = match to {
        TemperatureUnit::Celsius => temp.to_celsius(),
        TemperatureUnit::Fahrenheit => temp.to_fahrenheit(),
    };

    let converted_temp = Temperature::new(converted_value, to);
    
    println!(
        "{} = {}",
        temp.display(),
        converted_temp.display()
    );
}