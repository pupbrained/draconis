use {
    openweathermap::blocking::weather,
    std::{env::var, fs, process::Command, thread, time},
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

    println!("Hello, {}!", name.trim_matches('\"'));

    match &weather(
        location.trim_matches('\"'),
        units.trim_matches('\"'),
        lang.trim_matches('\"'),
        api_key.trim_matches('\"'),
    ) {
        Ok(current) => println!(
            "Right now in {}, it's a {} outside.",
            current.name.as_str(),
            current.weather[0].description.as_str(),
        ),
        Err(e) => panic!("Could not fetch weather because: {}", e),
    }

    let count = check_updates();

    match count {
        -1 => (),
        0 => println!("There are currently no updates available."),
        1 => println!("There is currently 1 update available."),
        _ => println!("There are currently {} updates available.", count),
    }

    println!("Here's your fetch:\n");

    Command::new("neofetch")
        .spawn()
        .expect("Failed to run fetch command.");

    let sleep = time::Duration::from_millis(250);
    thread::sleep(sleep);
}
