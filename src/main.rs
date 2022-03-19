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
    let time_format = json.get("time_format").expect("Couldn't find 'time_format' attribute.").to_string();
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
        match dt.hour() {
            0 | 12 => println!("🕛 {}", time.trim_start_matches(' ')),
            1 | 13 => println!("🕐 {}", time.trim_start_matches(' ')),
            2 | 14 => println!("🕑 {}", time.trim_start_matches(' ')),
            3 | 15 => println!("🕒 {}", time.trim_start_matches(' ')),
            4 | 16 => println!("🕓 {}", time.trim_start_matches(' ')),
            5 | 17 => println!("🕔 {}", time.trim_start_matches(' ')),
            6 | 18 => println!("🕕 {}", time.trim_start_matches(' ')),
            7 | 19 => println!("🕖 {}", time.trim_start_matches(' ')),
            8 | 20 => println!("🕗 {}", time.trim_start_matches(' ')),
            9 | 21 => println!("🕘 {}", time.trim_start_matches(' ')),
            10 | 22 => println!("🕙 {}", time.trim_start_matches(' ')),
            11 | 23 => println!("🕚 {}", time.trim_start_matches(' ')),
            _ => (),
        }
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
            println!(
                "☁️ {} {}°{}",
                current.weather[0].main.as_str(),
                current.main.temp.to_string().substring(0, 2),
                deg
            )
        }
        Err(e) => panic!("Could not fetch weather because: {}", e),
    }

    let count = check_updates();

    match count {
        -1 => (),
        0 => println!("📦 No updates"),
        1 => println!("📦 1 update"),
        _ => println!("📦 {} updates", count),
    }

    println!();

    Exec::cmd("neofetch").join().expect("Failed to run fetch!");
}
