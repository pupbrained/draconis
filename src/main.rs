use {
    chrono::prelude::*,
    openweathermap::blocking::weather,
    std::{env::var, fs},
    subprocess::*,
    substring::Substring,
};

fn read_config() -> serde_json::Value {
    let path = format!("{}/.config/hello-rs/config.json", var("HOME").unwrap());
    let file = fs::File::open(path).expect("Failed to open config file.");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("Failed to parse config file as a JSON.");
    json
}

fn check_updates() -> i32 {
    let mut total_updates = 0;

    let json = read_config();

    if json["package_managers"] == serde_json::json![null] {
        return -1;
    }

    let pm = json["package_managers"].as_array().unwrap();

    if pm.len() == 1 {
        match pm[0].to_string().trim_matches('\"') {
            "pacman" => {
                let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "apt" => {
                let update_count = {
                    Exec::cmd("apt-get").arg("upgrade").arg("-s")
                        | Exec::cmd("grep").arg("-P").arg("^\\d+ upgraded")
                        | Exec::cmd("cut").arg("-d").arg(" ").arg("-f1")
                }
                .capture()
                .unwrap()
                .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "xbps" => {
                let update_count =
                    { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            "portage" => {
                let update_count = {
                    Exec::cmd("eix").arg("--installed").arg("--upgrade")
                        | Exec::cmd("grep").arg("-P").arg("Found \\d+ matches")
                        | Exec::cmd("cut").arg("-d").arg(" ").arg("-f2")
                }
                .capture()
                .unwrap()
                .stdout_str();
                total_updates += update_count
                    .trim_end_matches('\n')
                    .parse::<i32>()
                    .unwrap_or(0);
            }
            "apk" => {
                let update_count =
                    { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
            }
            _ => (),
        }
    } else {
        for i in 0..pm.len() {
            match pm[i].to_string().trim_matches('\"') {
                "pacman" => {
                    let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                    total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
                }
                "apt" => {
                    let update_count = {
                        Exec::cmd("apt-get").arg("upgrade").arg("-s")
                            | Exec::cmd("grep").arg("-P").arg("^\\d+ upgraded")
                            | Exec::cmd("cut").arg("-d").arg(" ").arg("-f1")
                    }
                    .capture()
                    .unwrap()
                    .stdout_str();
                    total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
                }
                "xbps" => {
                    let update_count =
                        { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                            .capture()
                            .unwrap()
                            .stdout_str();
                    total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
                }
                "portage" => {
                    let update_count = {
                        Exec::cmd("eix").arg("--installed").arg("--upgrade")
                            | Exec::cmd("grep").arg("-P").arg("Found \\d+ matches")
                            | Exec::cmd("cut").arg("-d").arg(" ").arg("-f2")
                    }
                    .capture()
                    .unwrap()
                    .stdout_str();
                    total_updates += update_count
                        .trim_end_matches('\n')
                        .parse::<i32>()
                        .unwrap_or(0);
                }
                "apk" => {
                    let update_count =
                        { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                            .capture()
                            .unwrap()
                            .stdout_str();
                    total_updates += update_count.trim_end_matches('\n').parse::<i32>().unwrap();
                }
                _ => (),
            }
        }
    };
    total_updates
}

fn uppercase(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn main() {
    let json = read_config();
    let name = json
        .get("name")
        .expect("Couldn't find 'name' attribute.")
        .to_string();
    let location = json
        .get("location")
        .expect("Couldn't find 'location' attribute.")
        .to_string();
    let units = json
        .get("units")
        .expect("Couldn't find 'units' attribute.")
        .to_string();
    let lang = json
        .get("lang")
        .expect("Couldn't find 'lang' attribute.")
        .to_string();
    let api_key = json
        .get("api_key")
        .expect("Couldn't find 'api_key' attribute.")
        .to_string();
    let time_format = json
        .get("time_format")
        .expect("Couldn't find 'time_format' attribute.")
        .to_string();
    let dt = Local::now();
    let time = if time_format.trim_matches('\"') == "12h" {
        dt.format("%l:%M %p").to_string()
    } else if time_format.trim_matches('\"') == "24h" {
        dt.format("%H:%M").to_string()
    } else {
        "off".to_string()
    };

    match dt.hour() {
        6..=11 => println!("🌇 Good morning, {}!", name.trim_matches('\"')),
        12..=17 => println!("🏙️ Good afternoon, {}!", name.trim_matches('\"')),
        18..=22 => println!("🌆 Good evening, {}!", name.trim_matches('\"')),
        _ => println!("🌃 Good night, {}!", name.trim_matches('\"')),
    }

    if time != "off" {
        let time_icon;
        match dt.hour() {
            0 | 12 => time_icon = "🕛",
            1 | 13 => time_icon = "🕐",
            2 | 14 => time_icon = "🕑",
            3 | 15 => time_icon = "🕒",
            4 | 16 => time_icon = "🕓",
            5 | 17 => time_icon = "🕔",
            6 | 18 => time_icon = "🕕",
            7 | 19 => time_icon = "🕖",
            8 | 20 => time_icon = "🕗",
            9 | 21 => time_icon = "🕘",
            10 | 22 => time_icon = "🕙",
            11 | 23 => time_icon = "🕚",
            _ => time_icon = "🕛",
        }
        println!("{} {}", time_icon, time.trim_start_matches(' '));
    }

    match &weather(
        location.trim_matches('\"'),
        units.trim_matches('\"'),
        lang.trim_matches('\"'),
        api_key.trim_matches('\"'),
    ) {
        Ok(current) => {
            let deg = if units.trim_matches('\"') == "imperial" {
                "F"
            } else {
                "C"
            };
            let icon_code = &current.weather[0].icon;
            let icon;
            match icon_code.as_ref() {
                "01d" => icon = "☀️",
                "01n" => icon = "🌙",
                "02d" => icon = "⛅️",
                "02n" => icon = "🌙",
                "03d" => icon = "☁️",
                "03n" => icon = "☁️",
                "04d" => icon = "☁️",
                "04n" => icon = "☁️",
                "09d" => icon = "🌧️",
                "09n" => icon = "🌧️",
                "10d" => icon = "🌧️",
                "10n" => icon = "🌧️",
                "11d" => icon = "⛈️",
                "11n" => icon = "⛈️",
                "13d" => icon = "🌨️",
                "13n" => icon = "🌨️",
                "40d" => icon = "🌫️",
                "40n" => icon = "🌫️",
                "50d" => icon = "🌫️",
                "50n" => icon = "🌫️",
                _ => icon = "❓",
            }
            println!(
                "{} {} {}°{}",
                icon,
                uppercase(current.weather[0].description.as_ref()),
                current.main.temp.to_string().substring(0, 2),
                deg
            )
        }
        Err(e) => panic!("Could not fetch weather because: {}", e),
    }

    let count = check_updates();

    match count {
        -1 => (),
        0 => println!("☑️ Up to date"),
        1 => println!("1️⃣ 1 update"),
        2 => println!("2️⃣ 2 updates"),
        3 => println!("3️⃣ 3 updates"),
        4 => println!("4️⃣ 4 updates"),
        5 => println!("5️⃣ 5 updates"),
        6 => println!("6️⃣ 6 updates"),
        7 => println!("7️⃣ 7 updates"),
        8 => println!("8️⃣ 8 updates"),
        9 => println!("9️⃣ 9 updates"),
        10 => println!("🔟 10 updates"),
        _ => println!("‼️ {} updates", count),
    }

    println!();

    Exec::cmd("neofetch").join().expect("Failed to run fetch!");
}
