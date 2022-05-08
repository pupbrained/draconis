use {
    std::io::{stdout, Write},
    text_io::read,
};

pub(crate) fn make_config() {
    println!("Creating a new config file...");
    ask("What would you like to use for your hostname? (Leave blank for the default): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "" => println!("Using the default hostname..."),
        _ => {
            println!("Using {} as the hostname...", ans);
            let hostname = ans;
        }
    }
    ask("What would you like to use for your name? (Leave blank for the default): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "" => println!("Using the default name..."),
        _ => {
            println!("Using {} as the name...", ans);
            let name = ans;
        }
    }
    ask("Would you like to enable greetings? (y/n): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "y" | "Y" => {
            println!("Enabling...");
            let greeting = true;
        }
        "n" | "N" => {
            println!("Disabling...");
            let greeting = false;
        }
        _ => println!("Invalid input, disabling..."),
    }
    ask("Would you like to enable icons? (y/n): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "y" | "Y" => {
            println!("Enabling...");
            let icons = true;
            println!(
                "What icon type would you like to use? (emoji or normal, leave blank for normal): "
            );
            let ans: String = read!("{}\n");
            match ans.as_str().to_lowercase().as_str() {
                "emoji" => {
                    println!("Using emoji...");
                    let icon_type = "emoji";
                }
                "normal" => {
                    println!("Using normal...");
                    let icon_type = "normal";
                }
                _ => {
                    println!("Using normal...");
                    let icon_type = "normal";
                }
            }
        }
        "n" | "N" => {
            println!("Disabling...");
            let icons = false;
        }
        _ => {
            println!("Invalid input, disabling...");
            let icons = false;
        }
    }
    ask("Would you like to enable showing the time? (y/n): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "y" | "Y" => {
            println!("Enabling...");
            let time = true;
            ask("What time format would you like to use? (12 or 24, leave blank for 12): ");
            let ans: String = read!("{}\n");
            match ans.as_str().to_lowercase().as_str() {
                "12" => {
                    println!("Using 12 hour time...");
                    let time_format = "12";
                }
                "24" => {
                    println!("Using 24 hour time...");
                    let time_format = "24";
                }
                _ => {
                    println!("Using 12 hour time...");
                    let time_format = "12";
                }
            }
        }
        "n" | "N" => {
            println!("Disabling...");
            let time = false;
        }
        _ => {
            println!("Invalid input, disabling...");
            let time = false;
        }
    }
    ask("Would you like to enable weather? (y/n): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "y" | "Y" => {
            let weather = true;
            ask("What weather format would you like to use? (metric or imperial, leave blank for metric): ");
            let ans: String = read!("{}\n");
            match ans.as_str().to_lowercase().as_str() {
                "metric" | _ => {
                    println!("Using metric...");
                    let weather_format = "metric";
                }
                "imperial" => {
                    println!("Using imperial...");
                    let weather_format = "imperial";
                }
            }
            ask("What's your API key? You can get one from openweathermap (Leave blank for the default): ");
            let ans: String = read!("{}\n");
            println!("Using {} as the API key...", ans);
            let api_key = ans;
            ask("What's your preferred language? (Use the two-letter code, or leave blank for en): ");
            let ans: String = read!("{}\n");
            match ans.as_str().to_lowercase().as_str() {
                "en" | "" => {
                    println!("Using English...");
                    let language = "en";
                }
                _ => {
                    println!("Using {}...", ans);
                    let language = ans;
                }
            }
            ask("What's your city? You can also enter an openweathermap city ID (Leave blank for the default): ");
            let ans: String = read!("{}\n");
            println!("Using {} as the city...", ans);
            let city = ans;
        }
        "n" | "N" => {
            println!("Disabling...");
            let weather = false;
        }
        _ => {
            println!("Invalid input, disabling...");
            let weather = false;
        }
    }
    ask("Would you like to enable showing your OS name? (y/n): ");
    let ans: String = read!("{}\n");
    match ans.as_str() {
        "y" | "Y" => {
            println!("Enabling...");
            let os = true;
        }
        "n" | "N" => {
            println!("Disabling...");
            let os = false;
        }
        _ => {
            println!("Invalid input, disabling...");
            let os = false;
        }
    }
}

fn ask(msg: &str) {
    print!("{}", msg);
    stdout().flush().unwrap();
}
