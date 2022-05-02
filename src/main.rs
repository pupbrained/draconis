use {
    argparse::{ArgumentParser, Store},
    chrono::prelude::{Local, Timelike},
    mpris::PlayerFinder,
    once_cell::sync::Lazy,
    openweathermap::weather,
    std::{env, fs::File, io::ErrorKind, process::Stdio, time::Instant},
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
    whoami::{realname, username},
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

    let file = match File::open(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => return serde_json::json!({}),
        Err(e) => panic!("{}", e),
        Ok(file) => file,
    };

    serde_json::from_reader(file).unwrap()
}

fn check_update_commmand(command: String) -> Option<(CommandKind, Command)> {
    let tup = match command.as_str() {
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
        other => {
            tracing::warn!("Unsupported package manager: {}", other);
            return None;
        }
    };

    Some(tup)
}

async fn count_lines(skip: i32, mut reader: BufReader<ChildStdout>) -> Option<i32> {
    let mut total = 0;
    let mut s = String::new();

    loop {
        let n = reader
            .read_line(&mut s)
            .await
            .map_err(|e| tracing::warn!("Failed to read line from command output, {}", e))
            .ok()?;

        if n == 0 {
            break;
        }
        s.clear();
        total += 1;
    }

    if total > skip {
        Some(total - skip)
    } else {
        Some(0)
    }
}

#[tracing::instrument]
async fn do_update_counting(arg: String) -> Option<i32> {
    let (kind, mut command) = check_update_commmand(arg)?;
    let reader = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?
        .stdout
        .take()?;

    let fs = BufReader::new(reader);
    match kind {
        CommandKind::Apt => count_lines(2, fs).await,
        CommandKind::Portage => 0, // FIXME: Portage needs a proper update count command
        CommandKind::Dnf => count_lines(3, fs).await,
        _ => count_lines(0, fs).await,
    }
}

async fn check_updates() -> Option<i32> {
    match &JSON["package_managers"] {
        serde_json::Value::Array(pm) => {
            let mut handles = Vec::new();

            for arg in pm {
                if let serde_json::Value::String(string) = arg {
                    let handle = tokio::spawn(do_update_counting(string.clone()));
                    handles.push(handle);
                }
            }

            let mut total_updates = 0;

            for handle in handles {
                total_updates += handle.await.ok()??;
            }

            Some(total_updates)
        }
        serde_json::Value::String(string) => do_update_counting(string.clone()).await,
        _ => None,
    }
}

fn check_installed_command(command: String) -> Option<(CommandKind, Command)> {
    let tup = match command.as_str() {
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
            let mut command = Command::new("qlist");
            command.arg("-I");
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
        other => {
            tracing::warn!("unknown package manager: {}", other);
            return None;
        }
    };

    Some(tup)
}

#[tracing::instrument]
async fn do_installed_counting(arg: String) -> Option<i32> {
    let (kind, mut command) = check_installed_command(arg)?;
    let reader = command
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?
        .stdout
        .take()?;

    let fs = BufReader::new(reader);
    match kind {
        CommandKind::Apt => count_lines(2, fs).await,
        _ => count_lines(0, fs).await,
    }
}

#[tracing::instrument]
async fn get_package_count() -> Option<i32> {
    match &JSON["package_managers"] {
        serde_json::Value::Array(pm) => {
            let mut handles = Vec::new();

            for arg in pm {
                if let serde_json::Value::String(string) = arg {
                    let handle = tokio::spawn(do_installed_counting(string.to_owned()));
                    handles.push(handle);
                }
            }

            let mut total_packages = 0;

            for handle in handles {
                total_packages += handle.await.ok()??;
            }

            Some(total_packages)
        }
        serde_json::Value::String(string) => do_installed_counting(string.clone()).await,
        _ => None,
    }
}

#[tracing::instrument]
fn get_release_blocking() -> Option<String> {
    let rel = linux_os_release().ok()?.pretty_name?; // this performs a blocking read of /etc/os-release

    if rel.len() > 41 {
        Some(format!("{}...", rel.trim_matches('\"').substring(0, 37)))
    } else {
        Some(
            rel.trim_matches('\"')
                .trim_end_matches('\n')
                .trim_end_matches('\"')
                .to_string(),
        )
    }
}

#[tracing::instrument]
fn get_kernel_blocking() -> Option<String> {
    let kernel = os_release().ok()?; // this performs a blocking read of /proc/sys/kernel/osrelease
    if kernel.len() > 41 {
        Some(format!("{}...", kernel.substring(0, 37)))
    } else {
        Some(kernel.trim_end_matches('\n').to_string())
    }
}

#[tracing::instrument]
fn get_song() -> Option<String> {
    if JSON["song"] == false || JSON["song"].is_null() {
        return None;
    }

    let player = PlayerFinder::new()
        .ok()?
        .find_active() // this is blocking
        .ok()?;
    let song = player.get_metadata().ok()?; // this is blocking
    let songname = format!("{} - {}", song.artists()?.first()?, song.title()?);

    if songname.len() > 41 {
        Some(format!("{}...", songname.substring(0, 37)))
    } else {
        Some(songname.trim_end_matches('\n').to_string())
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
async fn get_weather() -> Option<String> {
    if JSON["location"].is_null()
        || JSON["units"].is_null()
        || JSON["lang"].is_null()
        || JSON["api_key"].is_null()
    {
        return None;
    }

    let location = JSON.get("location")?.as_str()?;
    let units = JSON.get("units")?.as_str()?;
    let lang = JSON.get("lang")?.as_str()?;
    let api_key = JSON.get("api_key")?.as_str()?;

    match &weather(location, units, lang, api_key).await {
        Ok(current) => {
            let deg = if units.trim_matches('\"') == "imperial" {
                "F"
            } else {
                "C"
            };
            let icon_code = &current.weather[0].icon;
            let icon = match icon_code.as_ref() {
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
            let main = current.weather[0].main.to_string();
            let temp = current.main.temp.to_string();

            Some(format!(
                "│ {} {} {}°{}",
                icon,
                main,
                temp.substring(0, 2),
                deg
            ))
        }
        Err(e) => {
            tracing::warn!("Could not fetch weather because: {}", e);
            None
        }
    }
}

#[tracing::instrument]
fn greeting() -> Option<String> {
    let name = if JSON["name"] == serde_json::json![null] {
        realname()
    } else {
        JSON.get("name")?.as_str()?.to_owned()
    };

    let phrase = match Local::now().hour() {
        6..=11 => "🌇 Good morning",
        12..=17 => "🏙️ Good afternoon",
        18..=22 => "🌆 Good evening",
        _ => "🌃 Good night",
    };

    Some(format!("{}, {}", phrase, name))
}

#[tracing::instrument]
fn get_hostname() -> Option<String> {
    if let serde_json::Value::String(string) = &JSON["hostname"] {
        return Some(string.clone());
    }

    Some(format!("{}@{}", username(), hostname().ok()?))
}

#[tracing::instrument]
fn get_datetime() -> Option<String> {
    let dt = Local::now();
    let time = match &JSON["time_format"] {
        serde_json::Value::String(time) => match time.as_str() {
            "12h" => dt.format("%l:%M %p").to_string(),
            "24h" => dt.format("%H:%M").to_string(),
            _ => "off".to_string(),
        },
        _ => return None,
    };
    let day = dt.format("%e").to_string();
    let date = match day.trim_start_matches(' ') {
        "1" | "21" | "31 " => format!("{} {}st", dt.format("%B"), day.trim_start_matches(' ')),
        "2" | "22" => format!("{} {}nd", dt.format("%B"), day.trim_start_matches(' ')),
        "3" | "23" => format!("{} {}rd", dt.format("%B"), day.trim_start_matches(' ')),
        _ => format!("{} {}th", dt.format("%B"), day.trim_start_matches(' ')),
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
    Some(format!(
        "│ {} {}, {}",
        time_icon,
        date,
        time.trim_start_matches(' ')
    ))
}

#[tracing::instrument]
async fn count_updates() -> Option<String> {
    let count = check_updates().await?;
    let update_count;
    let updates: String = match count {
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
    Some(format!("│ {}", updates))
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

    if let Some(hostname) = hostname {
        println!(
            "{}",
            calc_with_hostname(format!("╭─\x1b[32m{}\x1b[0m", hostname))
        );
    }

    if let Some(greeting) = greeting {
        println!("{}", calc_whitespace(format!("│ {}!", greeting)));
    }

    if let Some(datetime) = datetime {
        println!("{}", calc_whitespace(datetime));
    }

    if let Some(weather) = weather {
        println!("{}", calc_whitespace(weather));
    }

    if let Some(release) = release {
        println!("{}", calc_whitespace(format!("│ 💻 {}", release)));
    }
    if let Some(kernel) = kernel {
        println!("{}", calc_whitespace(format!("│ 🫀 {}", kernel)));
    }
    println!("{}", calc_whitespace(format!("│ 🧠 {}", memory)));
    println!("{}", calc_whitespace(format!("│ 💾 {}", disk)));

    match environment.as_ref() {
        "" => (),
        _ => println!(
            "{}",
            calc_whitespace(format!("│ 🖥️ {}", upper_first(environment)))
        ),
    }

    if let Some(count) = up_count {
        println!("{}", calc_whitespace(count));
    }

    match package_count {
        None => (),
        Some(0) => println!("{}", calc_whitespace("│ 📦 No packages".to_string())),
        Some(1) => println!("{}", calc_whitespace("│ 📦 1 package".to_string())),
        Some(n) => println!("{}", calc_whitespace(format!("│ 📦 {} packages", n))),
    }

    if let Some(song) = song.as_ref() {
        println!(
            "{}",
            calc_whitespace(format!("│ 🎵 {}", song.trim_matches('\n')))
        );
    }

    println!("╰─────────────────────────────────────────────╯");
}
