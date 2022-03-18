use {
    openweathermap::blocking::weather,
    std::{
        env::var,
        fs,
        io::{Read, Write},
        process::{Command, Stdio},
        thread, time,
    },
};

fn read_config() -> serde_json::Value {
    let path = format!("{}/.config/hello-rs/config.json", var("HOME").unwrap());
    let file = fs::File::open(path)
        .expect("Failed to open config file.");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("Failed to parse config file as a JSON.");
    json
}

fn check_updates() -> i32 {
    let mut update_check = Command::new("checkupdates")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut word_count = Command::new("wc")
        .arg("-l")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(ref mut stdout) = update_check.stdout {
        if let Some(ref mut stdin) = word_count.stdin {
            let mut buf: Vec<u8> = Vec::new();
            stdout.read_to_end(&mut buf).unwrap();
            stdin.write_all(&buf).unwrap();
        }
    }

    let res = word_count
        .wait_with_output()
        .unwrap()
        .stdout
        .to_ascii_uppercase();

    let s = match std::str::from_utf8(&res) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    s.trim_end_matches('\n').parse().unwrap()
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
