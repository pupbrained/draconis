use whoami::realname;

use {
    argparse::{ArgumentParser, Store},
    chrono::prelude::{Local, Timelike},
    mpris::PlayerFinder,
    once_cell::sync::Lazy,
    openweathermap::weather,
    std::{env, fs::File, process::Stdio, time::Instant},
    substring::Substring,
    sys_info::{hostname, linux_os_release, os_release},
    systemstat::{saturating_sub_bytes, Platform, System},
    tokio::{
        io::{AsyncBufReadExt, BufReader},
        process::{ChildStdout, Command},
    },
    tracing_subscriber::{
        fmt::{self, format::FmtSpan},
        prelude::*,
        EnvFilter,
    },
    unicode_segmentation::UnicodeSegmentation,
    whoami::username,
};

#[derive(Debug)]
enum CommandKind {
    Pacman,
    Apt,
    Xbps,
    Portage,
    Apk,
    Dnf,
}

static JSON: Lazy<serde_json::Value> = Lazy::new(read_config);

fn read_config() -> serde_json::Value {
    let mut path = format!("{}/.config/hello-rs/config.json", env::var("HOME").unwrap());
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("A simple greeter for your terminal, made in Rust");
        ap.refer(&mut path).add_option(
            &["-c", "--config"],
            Store,
            "Specify a path to a config file",
        );
        ap.parse_args_or_exit();
    }
    serde_json::from_reader(File::open(path).expect("Failed to open config file."))
        .expect("Failed to parse config file as a JSON.")
}

fn check_update_commmand(command: String) -> (CommandKind, Command) {
    match command.trim_matches('\"') {
        "pacman" => (CommandKind::Pacman, Command::new("checkupdates")),
        "apt" => (CommandKind::Apt, {
            let mut command = Command::new("apt");
            command.args(&["list", "-u"]);

            command
        }),
        "xbps" => (CommandKind::Xbps, {
            let mut command = Command::new("xbps-install");
            command.arg("-Sun");
            command
        }),
        "portage" => (CommandKind::Portage, {
            let mut command = Command::new("eix");
            command.args(&["-u", "--format", "'<installedversions:nameversion>'"]);
            command
        }),
        "apk" => (CommandKind::Apk, {
            let mut command = Command::new("apk");
            command.args(&["-u", "list"]);
            command
        }),
        "dnf" => (CommandKind::Dnf, {
            let mut command = Command::new("dnf");
            command.arg("check-update");
            command
        }),
        other => panic!("Unsupported package manager: {}", other),
    }
}

async fn count_lines(skip: i32, mut reader: BufReader<ChildStdout>) -> i32 {
    let mut total = 0;
    let mut s = String::new();

    loop {
        let n = reader.read_line(&mut s).await.unwrap();
        if n == 0 {
            break;
        }
        s.clear();
        total += 1;
    }

    if total > skip {
        total - skip
    } else {
        0
    }
}

#[tracing::instrument]
async fn do_update_counting(arg: String) -> i32 {
    let (kind, mut command) = check_update_commmand(arg);
    let reader = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .take()
        .unwrap();

    let fs = BufReader::new(reader);
    match kind {
        CommandKind::Apt => count_lines(2, fs).await,
        CommandKind::Portage => count_lines(0, fs).await,
        CommandKind::Dnf => count_lines(3, fs).await,
        _ => count_lines(0, fs).await,
    }
}

async fn check_updates() -> i32 {
    if JSON["package_managers"] == serde_json::json![null] {
        return -1;
    }

    if JSON["package_managers"].is_array() {
        let pm = JSON["package_managers"].as_array().unwrap();
        let mut handles = Vec::new();

        for arg in pm {
            let handle = tokio::spawn(do_update_counting(arg.to_string()));
            handles.push(handle);
        }

        let mut total_updates = 0;

        for handle in handles {
            total_updates += handle.await.unwrap();
        }

        total_updates
    } else {
        let pm = &JSON["package_managers"];
        do_update_counting(pm.to_string()).await
    }
}

fn check_installed_command(command: String) -> (CommandKind, Command) {
    match command.trim_matches('\"') {
        "pacman" => (CommandKind::Pacman, {
            let mut command = Command::new("pacman");
            command.arg("-Q");
            command
        }),
        "apt" => (CommandKind::Apt, {
            let mut command = Command::new("apt");
            command.args(&["list", "-i"]);
            command
        }),
        "xbps" => (CommandKind::Xbps, {
            let mut command = Command::new("xbps-query");
            command.arg("-l");
            command
        }),
        "portage" => (CommandKind::Portage, {
            let mut command = Command::new("eix-installed");
            command.arg("-a");
            command
        }),
        "apk" => (CommandKind::Apk, {
            let mut command = Command::new("apk");
            command.arg("info");
            command
        }),
        "dnf" => (CommandKind::Dnf, {
            let mut command = Command::new("dnf");
            command.args(&["list", "installed"]);
            command
        }),
        other => panic!("unknown package manager: {}", other),
    }
}

#[tracing::instrument]
async fn do_installed_counting(arg: String) -> i32 {
    let (kind, mut command) = check_installed_command(arg);
    let reader = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .take()
        .unwrap();

    let fs = BufReader::new(reader);
    match kind {
        CommandKind::Apt => count_lines(2, fs).await,
        CommandKind::Portage => count_lines(0, fs).await,
        _ => count_lines(0, fs).await,
    }
}

#[tracing::instrument]
async fn get_package_count() -> i32 {
    if JSON["package_managers"] == serde_json::json![null] {
        return -1;
    }

    if JSON["package_managers"].is_array() {
        let pm = JSON["package_managers"].as_array().unwrap();
        let mut handles = Vec::new();

        for arg in pm {
            let handle = tokio::spawn(do_installed_counting(arg.to_string()));
            handles.push(handle);
        }

        let mut total_packages = 0;

        for handle in handles {
            total_packages += handle.await.unwrap();
        }

        total_packages
    } else {
        let pm = &JSON["package_managers"];
        do_installed_counting(pm.to_string()).await
    }
}

#[tracing::instrument]
fn get_release_blocking() -> String {
    let rel = linux_os_release().unwrap().pretty_name.unwrap(); // this performs a blocking read of /etc/os-release

    if rel.len() > 41 {
        format!("{}...", rel.trim_matches('\"').substring(0, 37))
    } else {
        rel.trim_matches('\"')
            .trim_end_matches('\n')
            .trim_end_matches('\"')
            .to_string()
    }
}

#[tracing::instrument]
fn get_kernel_blocking() -> String {
    let kernel = os_release().unwrap(); // this performs a blocking read of /proc/sys/kernel/osrelease
    if kernel.len() > 41 {
        format!("{}...", kernel.substring(0, 37))
    } else {
        kernel.trim_end_matches('\n').to_string()
    }
}

#[tracing::instrument]
fn get_song() -> String {
    if JSON["song"] == false || JSON["song"] == serde_json::json![null] {
        return "".to_string();
    }
    let player = PlayerFinder::new()
        .expect("Could not connect to DBus")
        .find_active(); // this is blocking
    let player = match player {
        Ok(p) => p,
        Err(_) => return "".to_string(),
    };
    let song = player.get_metadata().expect("Failed to get metadata"); // this is blocking
    let songname = format!(
        "{} - {}",
        song.artists().unwrap().first().unwrap(),
        song.title().unwrap()
    );
    if songname.len() > 41 {
        format!("{}...", songname.substring(0, 37))
    } else {
        songname.trim_end_matches('\n').to_string()
    }
}

fn upper_first(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn calc_whitespace(text: String) -> String {
    let size = 45 - text.graphemes(true).count();
    let fs = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, fs)
}

fn calc_with_hostname(text: String) -> String {
    let size = 55 - text.graphemes(true).count();
    let fs = format!("{}{}", "─".repeat(size), "╮");
    format!("{}{}", text, fs)
}

#[tracing::instrument]
fn get_environment() -> String {
    env::var::<String>(ToString::to_string(&"XDG_CURRENT_DESKTOP"))
        .unwrap_or_else(|_| env::var(&"XDG_SESSION_DESKTOP").unwrap_or_else(|_| "".to_string()))
}

#[tracing::instrument]
async fn get_weather() -> String {
    if JSON["location"] == serde_json::json![null]
        || JSON["units"] == serde_json::json![null]
        || JSON["lang"] == serde_json::json![null]
        || JSON["api_key"] == serde_json::json![null]
    {
        return "".to_string();
    }
    let deg;
    let icon_code;
    let icon;
    let main;
    let temp;
    let location = JSON
        .get("location")
        .expect("Couldn't find 'location' attribute.")
        .to_string();
    let units = JSON
        .get("units")
        .expect("Couldn't find 'units' attribute.")
        .to_string();
    let lang = JSON
        .get("lang")
        .expect("Couldn't find 'lang' attribute.")
        .to_string();
    let api_key = JSON
        .get("api_key")
        .expect("Couldn't find 'api_key' attribute.")
        .to_string();
    match &weather(
        location.trim_matches('\"'),
        units.trim_matches('\"'),
        lang.trim_matches('\"'),
        api_key.trim_matches('\"'),
    )
    .await
    {
        Ok(current) => {
            deg = if units.trim_matches('\"') == "imperial" {
                "F"
            } else {
                "C"
            };
            icon_code = &current.weather[0].icon;
            icon = match icon_code.as_ref() {
                "01d" => "☀️",
                "01n" => "🌙",
                "02d" => "⛅️",
                "02n" => "🌙",
                "03d" => "☁️",
                "03n" => "☁️",
                "04d" => "☁️",
                "04n" => "☁️",
                "09d" => "🌧️",
                "09n" => "🌧️",
                "10d" => "🌧️",
                "10n" => "🌧️",
                "11d" => "⛈️",
                "11n" => "⛈️",
                "13d" => "🌨️",
                "13n" => "🌨️",
                "40d" => "🌫️",
                "40n" => "🌫️",
                "50d" => "🌫️",
                "50n" => "🌫️",
                _ => "❓",
            };
            main = current.weather[0].main.to_string();
            temp = current.main.temp.to_string();
        }
        Err(e) => panic!("Could not fetch weather because: {}", e),
    }
    format!("│ {} {} {}°{}", icon, main, temp.substring(0, 2), deg)
}

#[tracing::instrument]
fn greeting() -> String {
    let name;
    if JSON["name"] == serde_json::json![null] {
        name = realname();
    } else {
        name = JSON
            .get("name")
            .expect("Couldn't find 'name' attribute.")
            .to_string();
    }
    match Local::now().hour() {
        6..=11 => "🌇 Good morning",
        12..=17 => "🏙️ Good afternoon",
        18..=22 => "🌆 Good evening",
        _ => "🌃 Good night",
    }
    .to_string()
        + ", "
        + name.trim_matches('\"')
}

#[tracing::instrument]
fn get_hostname() -> String {
    if JSON["hostname"] == serde_json::json![null] {
        return format!("{}@{}", username(), hostname().unwrap());
    }
    JSON.get("hostname")
        .unwrap()
        .to_string()
        .trim_matches('\"')
        .to_string()
}

#[tracing::instrument]
fn get_datetime() -> String {
    if JSON["time_format"] == serde_json::json![null] {
        return "".to_string();
    }
    let dt = Local::now();
    let day = dt.format("%e").to_string();
    let date = match day.trim_start_matches(' ') {
        "1" | "21" | "31 " => format!("{} {}st", dt.format("%B"), day.trim_start_matches(' ')),
        "2" | "22" => format!("{} {}nd", dt.format("%B"), day.trim_start_matches(' ')),
        "3" | "23" => format!("{} {}rd", dt.format("%B"), day.trim_start_matches(' ')),
        _ => format!("{} {}th", dt.format("%B"), day.trim_start_matches(' ')),
    };
    let time = match JSON
        .get("time_format")
        .expect("Couldn't find 'time_format' attribute.")
        .to_string()
        .trim_matches('\"')
    {
        "12h" => dt.format("%l:%M %p").to_string(),
        "24h" => dt.format("%H:%M").to_string(),
        _ => "off".to_string(),
    };
    let time_icon = match dt.hour() {
        0 | 12 => "🕛",
        1 | 13 => "🕐",
        2 | 14 => "🕑",
        3 | 15 => "🕒",
        4 | 16 => "🕓",
        5 | 17 => "🕔",
        6 | 18 => "🕕",
        7 | 19 => "🕖",
        8 | 20 => "🕗",
        9 | 21 => "🕘",
        10 | 22 => "🕙",
        11 | 23 => "🕚",
        _ => "🕛",
    };
    format!("│ {} {}, {}", time_icon, date, time.trim_start_matches(' '))
}

#[tracing::instrument]
async fn count_updates() -> String {
    let count = check_updates().await;
    let update_count;
    let updates: String = match count {
        -1 => "none",
        0 => "☑️ Up to date",
        1 => "1️⃣ 1 update",
        2 => "2️⃣ 2 updates",
        3 => "3️⃣ 3 updates",
        4 => "4️⃣ 4 updates",
        5 => "5️⃣ 5 updates",
        6 => "6️⃣ 6 updates",
        7 => "7️⃣ 7 updates",
        8 => "8️⃣ 8 updates",
        9 => "9️⃣ 9 updates",
        10 => "🔟 10 updates",
        _ => {
            update_count = format!("‼️ {} updates", count);
            update_count.as_ref()
        }
    }
    .to_string();
    format!("│ {}", updates)
}

#[tracing::instrument]
fn get_memory() -> String {
    match System::new().memory() {
        Ok(mem) => format!("{} Used", saturating_sub_bytes(mem.total, mem.free)),
        Err(x) => panic!("Could not get memory because: {}", x),
    }
}

#[tracing::instrument]
fn get_disk_usage() -> String {
    match System::new().mount_at("/") {
        Ok(disk) => {
            format!("{} Free", disk.free)
        }
        Err(x) => panic!("Could not get disk usage because: {}", x),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .init();

    tracing::info!("Running");

    let time = Instant::now();

    Lazy::force(&JSON);

    // These do not need to be spawned in any way, they are nonblocking
    let hostname = get_hostname();
    let greeting = greeting();
    let datetime = get_datetime();
    let memory = get_memory();
    let disk = get_disk_usage();
    let environment = get_environment();

    // These are proper async functions
    let weather = tokio::spawn(get_weather());
    let up_count = tokio::spawn(count_updates());
    let package_count = tokio::spawn(get_package_count());

    // These are functions that block
    let song = tokio::task::spawn_blocking(get_song);
    let release = tokio::task::spawn_blocking(get_release_blocking);
    let kernel = tokio::task::spawn_blocking(get_kernel_blocking);

    let weather = weather.await.unwrap();
    let up_count = up_count.await.unwrap();
    let package_count = package_count.await.unwrap();

    let song = song.await.unwrap();
    let release = release.await.unwrap();
    let kernel = kernel.await.unwrap();

    tracing::info!(
        "Finished collecting data in {:.3}",
        time.elapsed().as_secs_f32()
    );

    println!(
        "{}",
        calc_with_hostname(format!("╭─\x1b[32m{}\x1b[0m", hostname))
    );

    println!("{}", calc_whitespace(format!("│ {}!", greeting)));

    match datetime.as_ref() {
        "" => (),
        _ => println!("{}", calc_whitespace(datetime)),
    }

    match weather.as_ref() {
        "" => (),
        _ => println!("{}", calc_whitespace(weather)),
    }

    println!("{}", calc_whitespace(format!("│ 💻 {}", release)));
    println!("{}", calc_whitespace(format!("│ 🫀 {}", kernel)));
    println!("{}", calc_whitespace(format!("│ 🧠 {}", memory)));
    println!("{}", calc_whitespace(format!("│ 💾 {}", disk)));

    match environment.as_ref() {
        "" => (),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 🖥️ {}", upper_first(environment)))
        ),
    }

    if up_count != *"│ none".to_string() {
        println!("{}", calc_whitespace(up_count));
    }

    match package_count {
        -1 => (),
        0 => println!("{}", calc_whitespace("│ 📦 No packages".to_string())),
        1 => println!("{}", calc_whitespace("│ 📦 1 package".to_string())),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 📦 {} packages", package_count))
        ),
    }

    match song.as_ref() {
        "" => (),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 🎵 {}", song.trim_matches('\n')))
        ),
    }

    println!("╰─────────────────────────────────────────────╯");
}
