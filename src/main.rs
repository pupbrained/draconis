use {
    openweathermap::blocking::weather,
    std::{env::var, fs},
    subprocess::*,
    substring::Substring,
    chrono::prelude::*,
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
        if pm[0].to_string().trim_matches('\"') == "pacman" {
            let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                .capture()
                .unwrap()
                .stdout_str();
            total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
        }
        if pm[0].to_string().trim_matches('\"') == "apt" {
            let update_count = {
                Exec::cmd("apt-get").arg("upgrade").arg("-s")
                    | Exec::cmd("grep").arg("-P").arg("^\\d+ upgraded")
            }
            .capture()
            .unwrap()
            .stdout_str();
            total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
        }
        if pm[0].to_string().trim_matches('\"') == "xbps" {
            let update_count =
                { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
            total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
        }
        if pm[0].to_string().trim_matches('\"') == "portage" {
            let update_count = {
                Exec::cmd("eix").arg("--installed").arg("--upgrade")
                    | Exec::cmd("grep").arg("-P").arg("Found \\d+ matches")
            }
            .capture()
            .unwrap()
            .stdout_str();
            total_updates += update_count.substring(6, 7).parse::<i32>().unwrap();
        }
        if pm[0].to_string().trim_matches('\"') == "apk" {
            let update_count =
                { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
            total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
        }
    } else {
        for i in 0..pm.len() {
            if pm[i].to_string().trim_matches('\"') == "pacman" {
                let update_count = { Exec::cmd("checkupdates") | Exec::cmd("wc").arg("-l") }
                    .capture()
                    .unwrap()
                    .stdout_str();
                total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
            }
            if pm[i].to_string().trim_matches('\"') == "apt" {
                let update_count = {
                    Exec::cmd("apt-get").arg("upgrade").arg("-s")
                        | Exec::cmd("grep").arg("-P").arg("^\\d+ upgraded")
                }
                .capture()
                .unwrap()
                .stdout_str();
                total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
            }
            if pm[i].to_string().trim_matches('\"') == "xbps" {
                let update_count =
                    { Exec::cmd("xbps-install").arg("-Sun") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
            }
            if pm[i].to_string().trim_matches('\"') == "portage" {
                let update_count = {
                    Exec::cmd("eix").arg("--installed").arg("--upgrade")
                        | Exec::cmd("grep").arg("-P").arg("Found \\d+ matches")
                }
                .capture()
                .unwrap()
                .stdout_str();
                total_updates += update_count.substring(6, 7).parse::<i32>().unwrap_or(0);
            }
            if pm[i].to_string().trim_matches('\"') == "apk" {
                let update_count =
                    { Exec::cmd("apk").arg("-u").arg("list") | Exec::cmd("wc").arg("-l") }
                        .capture()
                        .unwrap()
                        .stdout_str();
                total_updates += update_count.substring(0, 1).parse::<i32>().unwrap();
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
    let dt = Local::now();
    let time = dt.hour(); 

    match time {
        6..=12 => println!("🌇 Good morning, {}!", name.trim_matches('\"')),
        13..=18 => println!("🏙️ Good afternoon, {}!", name.trim_matches('\"')),
        19..=23 => println!("🌆 Good evening, {}!", name.trim_matches('\"')),
        _ => println!("🌃 Good night, {}!", name.trim_matches('\"')),
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
