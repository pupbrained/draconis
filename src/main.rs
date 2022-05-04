pub mod statics;

use {
    argparse::{ArgumentParser, Store, StoreTrue},
    chrono::prelude::{Local, Timelike},
    mpris::PlayerFinder,
    once_cell::sync::Lazy,
    openweathermap::weather,
    serde::Deserialize,
    statics::*,
    std::{env, io::ErrorKind, process::Stdio, time::Instant},
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

#[derive(Deserialize)]
struct Config {
    main: Main,
    icons: Icons,
    time: Time,
    weather: Weather,
    packages: Packages,
    song: Song,
}

#[derive(Deserialize)]
struct Main {
    name: Option<String>,
}

#[derive(Deserialize)]
struct Icons {
    enabled: bool,
    kind: Option<String>,
}

#[derive(Deserialize)]
struct Time {
    kind: Option<String>,
}

#[derive(Deserialize)]
struct Weather {
    enabled: bool,
    values: WeatherValues,
}

#[derive(Deserialize)]
struct WeatherValues {
    api_key: Option<String>,
    location: Option<String>,
    lang: Option<String>,
    units: Option<String>,
}

#[derive(Deserialize)]
struct Packages {
    package_managers: Option<toml::Value>,
}

#[derive(Deserialize)]
struct Song {
    enabled: bool,
}

#[derive(Debug)]
enum CommandKind {
    Pacman,
    Apt,
    Xbps,
    Portage,
    Apk,
    Dnf,
}

fn read_config() -> Config {
    let mut path = format!("{}/.config/hello-rs/config.toml", env::var("HOME").unwrap());
    let mut ver = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("A simple greeter for your terminal, made in Rust");
        ap.refer(&mut path).add_option(
            &["-c", "--config"],
            Store,
            "Specify a path to a config file",
        );
        ap.refer(&mut ver)
            .add_option(&["-v", "--version"], StoreTrue, "View program version");
        ap.parse_args_or_exit();
    }

    if ver {
        println!("hello-rs v{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let content = match std::fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return toml::from_str(
                r#"
                [main]

                [icons]
                enabled = false

                [time]

                [weather]
                enabled = false

                [packages]

                [song]
                enabled = false
                "#,
            )
            .unwrap()
        }
        Err(e) => panic!("{}", e),
        Ok(content) => content,
    };

    toml::from_str(&content).unwrap()
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
        CommandKind::Portage => Some(0), // FIXME: Portage needs a proper update count command
        CommandKind::Dnf => count_lines(3, fs).await,
        _ => count_lines(0, fs).await,
    }
}

async fn check_updates() -> Option<i32> {
    match &CONF.packages.package_managers {
        Some(toml::Value::Array(pm)) => {
            let mut handles = Vec::new();

            for arg in pm {
                if let toml::Value::String(string) = arg {
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
        Some(toml::Value::String(pm)) => do_update_counting(pm.clone()).await,
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
    match &CONF.packages.package_managers {
        Some(toml::Value::Array(pm)) => {
            let mut handles = Vec::new();

            for arg in pm {
                if let toml::Value::String(string) = arg {
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
        Some(toml::Value::String(pm)) => do_installed_counting(pm.clone()).await,
        _ => None,
    }
}

#[tracing::instrument]
fn get_release_blocking() -> Option<String> {
    let rel = linux_os_release().ok()?.pretty_name?; // this performs a blocking read of /etc/os-release

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
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
            Some(&_) | None => {
                if rel.len() > 42 {
                    Some(format!("{}...", rel.trim_matches('\"').substring(0, 38)))
                } else {
                    Some(
                        rel.trim_matches('\"')
                            .trim_end_matches('\n')
                            .trim_end_matches('\"')
                            .to_string(),
                    )
                }
            }
        }
    } else if rel.len() > 42 {
        Some(format!("{}...", rel.trim_matches('\"').substring(0, 38)))
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
    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
                if kernel.len() > 41 {
                    Some(format!("{}...", kernel.substring(0, 37)))
                } else {
                    Some(kernel.trim_end_matches('\n').to_string())
                }
            }
            Some(&_) | None => {
                if kernel.len() > 42 {
                    Some(format!("{}...", kernel.substring(0, 38)))
                } else {
                    Some(kernel.trim_end_matches('\n').to_string())
                }
            }
        }
    } else if kernel.len() > 42 {
        Some(format!("{}...", kernel.substring(0, 38)))
    } else {
        Some(kernel.trim_end_matches('\n').to_string())
    }
}

#[tracing::instrument]
fn get_song() -> Option<String> {
    if !CONF.song.enabled {
        return None;
    }

    let player = PlayerFinder::new().ok()?.find_all().ok()?;
    let song = player.first()?.get_metadata().ok()?; // this is blocking
    let artists = song.artists()?.join(", ");
    let songname = format!("{} - {}", artists, song.title()?);

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
                if songname.len() > 41 {
                    Some(format!("{}...", songname.substring(0, 37)))
                } else {
                    Some(songname.trim_end_matches('\n').to_string())
                }
            }
            Some(&_) | None => {
                if songname.len() > 42 {
                    Some(format!("{}...", songname.substring(0, 38)))
                } else {
                    Some(songname.trim_end_matches('\n').to_string())
                }
            }
        }
    } else if songname.len() > 42 {
        Some(format!("{}...", songname.substring(0, 38)))
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
    let size = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => 55 - text.graphemes(true).count(),
            Some(&_) | None => 54 - text.graphemes(true).count(),
        }
    } else {
        54 - text.graphemes(true).count()
    };

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
    if CONF.weather.values.api_key.is_none()
        || CONF.weather.values.lang.is_none()
        || CONF.weather.values.location.is_none()
        || CONF.weather.values.units.is_none()
        || !CONF.weather.enabled
    {
        return None;
    }

    let api_key = CONF.weather.values.api_key.as_ref().unwrap().as_str();
    let lang = CONF.weather.values.lang.as_ref().unwrap().as_str();
    let location = CONF.weather.values.location.as_ref().unwrap().as_str();
    let units = CONF.weather.values.units.as_ref().unwrap().as_str();

    match &weather(location, units, lang, api_key).await {
        Ok(current) => {
            let deg = if units.trim_matches('\"') == "imperial" {
                "F"
            } else {
                "C"
            };
            let icon_code = &current.weather[0].icon;
            let icon = if CONF.icons.enabled {
                match CONF.icons.kind.as_deref() {
                    Some("emoji") => {
                        match icon_code.as_ref() {
                            "01d" => WEATHER_EMOJIS[0], // Clear sky
                            "01n" => WEATHER_EMOJIS[1],
                            "02d" => WEATHER_EMOJIS[2], // Few clouds
                            "02n" => WEATHER_EMOJIS[3],
                            "03d" => WEATHER_EMOJIS[4], // Scattered clouds
                            "03n" => WEATHER_EMOJIS[5],
                            "04d" => WEATHER_EMOJIS[6], // Broken clouds
                            "04n" => WEATHER_EMOJIS[7],
                            "09d" => WEATHER_EMOJIS[8], // Shower rain
                            "09n" => WEATHER_EMOJIS[9],
                            "10d" => WEATHER_EMOJIS[10], // Rain
                            "10n" => WEATHER_EMOJIS[11],
                            "11d" => WEATHER_EMOJIS[12], // Thunderstorm
                            "11n" => WEATHER_EMOJIS[13],
                            "13d" => WEATHER_EMOJIS[14], // Snow
                            "13n" => WEATHER_EMOJIS[15],
                            "40d" => WEATHER_EMOJIS[16], // Mist
                            "40n" => WEATHER_EMOJIS[17],
                            "50d" => WEATHER_EMOJIS[18], // Fog
                            "50n" => WEATHER_EMOJIS[19],
                            _ => WEATHER_EMOJIS[20], // Unknown
                        }
                    }
                    Some("normal") => match icon_code.as_ref() {
                        "01d" => WEATHER_ICONS[0],
                        "01n" => WEATHER_ICONS[1],
                        "02d" => WEATHER_ICONS[2],
                        "02n" => WEATHER_ICONS[3],
                        "03d" => WEATHER_ICONS[4],
                        "03n" => WEATHER_ICONS[5],
                        "04d" => WEATHER_ICONS[6],
                        "04n" => WEATHER_ICONS[7],
                        "09d" => WEATHER_ICONS[8],
                        "09n" => WEATHER_ICONS[9],
                        "10d" => WEATHER_ICONS[10],
                        "10n" => WEATHER_ICONS[11],
                        "11d" => WEATHER_ICONS[12],
                        "11n" => WEATHER_ICONS[13],
                        "13d" => WEATHER_ICONS[14],
                        "13n" => WEATHER_ICONS[15],
                        "40d" => WEATHER_ICONS[16],
                        "40n" => WEATHER_ICONS[17],
                        "50d" => WEATHER_ICONS[18],
                        "50n" => WEATHER_ICONS[19],
                        _ => WEATHER_ICONS[20],
                    },
                    Some(&_) | None => "",
                }
            } else {
                ""
            };

            let main = current.weather[0].main.to_string();
            let temp = current.main.temp.to_string();

            if CONF.icons.enabled {
                match CONF.icons.kind.as_deref() {
                    Some("emoji") | Some("normal") => Some(format!(
                        "│ {} {} {}°{}",
                        icon,
                        main,
                        temp.substring(0, 2),
                        deg
                    )),
                    Some(&_) | None => Some(format!(
                        "│{} {} {}°{}",
                        icon,
                        main,
                        temp.substring(0, 2),
                        deg
                    )),
                }
            } else {
                Some(format!(
                    "│{} {} {}°{}",
                    icon,
                    main,
                    temp.substring(0, 2),
                    deg
                ))
            }
        }
        Err(e) => {
            tracing::warn!(
                "Could not fetch weather because: {} - maybe you forgot an API key?",
                e
            );
            None
        }
    }
}

#[tracing::instrument]
fn greeting() -> Option<String> {
    let name = if CONF.main.name.is_none() {
        realname()
    } else {
        CONF.main.name.as_ref()?.to_string()
    };

    let phrase = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => match Local::now().hour() {
                6..=11 => format!("{} Good morning", GREET_EMOJIS[0]),
                12..=17 => format!("{} Good afternoon", GREET_EMOJIS[1]),
                18..=22 => format!("{} Good evening", GREET_EMOJIS[2]),
                _ => format!("{} Good night", GREET_EMOJIS[3]),
            },
            Some("normal") => match Local::now().hour() {
                6..=11 => format!("{} Good morning", GREET_ICONS[0]),
                12..=17 => format!("{} Good afternoon", GREET_ICONS[1]),
                18..=22 => format!("{} Good evening", GREET_ICONS[2]),
                _ => format!("{} Good night", GREET_ICONS[3]),
            },
            Some(&_) | None => match Local::now().hour() {
                6..=11 => "Good morning".to_string(),
                12..=17 => "Good afternoon".to_string(),
                18..=22 => "Good evening".to_string(),
                _ => "Good night".to_string(),
            },
        }
    } else {
        match Local::now().hour() {
            6..=11 => "Good morning".to_string(),
            12..=17 => "Good afternoon".to_string(),
            18..=22 => "Good evening".to_string(),
            _ => "Good night".to_string(),
        }
    };

    Some(format!("{}, {}", phrase, name))
}

#[tracing::instrument]
fn get_hostname() -> Option<String> {
    Some(format!("{}@{}", username(), hostname().ok()?))
}

#[tracing::instrument]
fn get_datetime() -> Option<String> {
    let dt = Local::now();
    let time = match CONF.time.kind.as_deref()? {
        "12h" => dt.format("%l:%M %p").to_string(),
        "24h" => dt.format("%H:%M").to_string(),
        _ => "off".to_string(),
    };
    let day = dt.format("%e").to_string();
    let date = match day.trim_start_matches(' ') {
        "1" | "21" | "31 " => format!("{} {}st", dt.format("%B"), day.trim_start_matches(' ')),
        "2" | "22" => format!("{} {}nd", dt.format("%B"), day.trim_start_matches(' ')),
        "3" | "23" => format!("{} {}rd", dt.format("%B"), day.trim_start_matches(' ')),
        _ => format!("{} {}th", dt.format("%B"), day.trim_start_matches(' ')),
    };
    let time_icon = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
                let index: usize = if dt.hour() > 12 {
                    (dt.hour() - 12).try_into().unwrap()
                } else {
                    dt.hour().try_into().unwrap()
                };
                TIME_EMOJIS[index.min(12)]
            }
            Some("normal") => {
                let index: usize = if dt.hour() > 12 {
                    (dt.hour() - 12).try_into().unwrap()
                } else {
                    dt.hour().try_into().unwrap()
                };
                TIME_ICONS[index.min(12)]
            }
            Some(&_) | None => "",
        }
    } else {
        ""
    };

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") | Some("normal") => Some(format!(
                "│ {} {}, {}",
                time_icon,
                date,
                time.trim_start_matches(' ')
            )),
            Some(&_) | None => Some(format!(
                "│{} {}, {}",
                time_icon,
                date,
                time.trim_start_matches(' ')
            )),
        }
    } else {
        Some(format!(
            "│{} {}, {}",
            time_icon,
            date,
            time.trim_start_matches(' ')
        ))
    }
}

#[tracing::instrument]
async fn count_updates() -> Option<String> {
    let count = check_updates().await?;
    let updates = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => match count {
                0 => format!("{} Up to date", PACKAGE_EMOJIS[0]),
                1 => format!("{} 1 update", PACKAGE_EMOJIS[1]),
                2 => format!("{} 2 updates", PACKAGE_EMOJIS[2]),
                3 => format!("{} 3 updates", PACKAGE_EMOJIS[3]),
                4 => format!("{} 4 updates", PACKAGE_EMOJIS[4]),
                5 => format!("{} 5 updates", PACKAGE_EMOJIS[5]),
                6 => format!("{} 6 updates", PACKAGE_EMOJIS[6]),
                7 => format!("{} 7 updates", PACKAGE_EMOJIS[7]),
                8 => format!("{} 8 updates", PACKAGE_EMOJIS[8]),
                9 => format!("{} 9 updates", PACKAGE_EMOJIS[9]),
                10 => format!("{} 10 updates", PACKAGE_EMOJIS[10]),
                _ => format!("{} {} updates", PACKAGE_EMOJIS[11], count),
            },
            Some("normal") => match count {
                0 => format!("{} Up to date", PACKAGE_ICONS[0]),
                1 => format!("{} 1 update", PACKAGE_ICONS[1]),
                2 => format!("{} 2 updates", PACKAGE_ICONS[2]),
                3 => format!("{} 3 updates", PACKAGE_ICONS[3]),
                4 => format!("{} 4 updates", PACKAGE_ICONS[4]),
                5 => format!("{} 5 updates", PACKAGE_ICONS[5]),
                6 => format!("{} 6 updates", PACKAGE_ICONS[6]),
                7 => format!("{} 7 updates", PACKAGE_ICONS[7]),
                8 => format!("{} 8 updates", PACKAGE_ICONS[8]),
                9 => format!("{} 9 updates", PACKAGE_ICONS[9]),
                _ => format!("{} {} updates", PACKAGE_ICONS[10], count),
            },
            Some(&_) | None => format!("{} updates", count),
        }
    } else {
        format!("{} updates", count)
    };
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
    get_song();
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

    Lazy::force(&CONF);

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

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
                if let Some(release) = release {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_EMOJIS[0], release))
                    );
                }
                if let Some(kernel) = kernel {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_EMOJIS[1], kernel))
                    );
                }
                println!(
                    "{}",
                    calc_whitespace(format!("│ {} {}", MISC_EMOJIS[2], memory))
                );
                println!(
                    "{}",
                    calc_whitespace(format!("│ {} {}", MISC_EMOJIS[3], disk))
                );

                match environment.as_ref() {
                    "" => (),
                    _ => println!(
                        "{}",
                        calc_whitespace(format!(
                            "│ {} {}",
                            MISC_EMOJIS[4],
                            upper_first(environment)
                        ))
                    ),
                }
            }
            Some("normal") => {
                if let Some(release) = release {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_ICONS[0], release))
                    );
                }
                if let Some(kernel) = kernel {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_ICONS[1], kernel))
                    );
                }
                println!(
                    "{}",
                    calc_whitespace(format!("│ {} {}", MISC_ICONS[2], memory))
                );
                println!(
                    "{}",
                    calc_whitespace(format!("│ {} {}", MISC_ICONS[3], disk))
                );

                match environment.as_ref() {
                    "" => (),
                    _ => println!(
                        "{}",
                        calc_whitespace(format!(
                            "│ {} {}",
                            MISC_ICONS[4],
                            upper_first(environment)
                        ))
                    ),
                }
            }
            Some(&_) | None => {
                if let Some(release) = release {
                    println!("{}", calc_whitespace(format!("│ {}", release)));
                }
                if let Some(kernel) = kernel {
                    println!("{}", calc_whitespace(format!("│ {}", kernel)));
                }
                println!("{}", calc_whitespace(format!("│ {}", memory)));
                println!("{}", calc_whitespace(format!("│ {}", disk)));

                match environment.as_ref() {
                    "" => (),
                    _ => println!(
                        "{}",
                        calc_whitespace(format!("│ {}", upper_first(environment)))
                    ),
                }
            }
        }
    } else {
        if let Some(release) = release {
            println!("{}", calc_whitespace(format!("│ {}", release)));
        }
        if let Some(kernel) = kernel {
            println!("{}", calc_whitespace(format!("│ {}", kernel)));
        }
        println!("{}", calc_whitespace(format!("│ {}", memory)));
        println!("{}", calc_whitespace(format!("│ {}", disk)));

        match environment.as_ref() {
            "" => (),
            _ => println!(
                "{}",
                calc_whitespace(format!("│ {}", upper_first(environment)))
            ),
        }
    }

    if let Some(count) = up_count {
        println!("{}", calc_whitespace(count));
    }

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => match package_count {
                None => (),
                Some(0) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} No packages", PACKAGE_EMOJIS[12]))
                ),
                Some(1) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} 1 package", PACKAGE_EMOJIS[12]))
                ),
                Some(n) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} {} packages", PACKAGE_EMOJIS[12], n))
                ),
            },
            Some("normal") => match package_count {
                None => (),
                Some(0) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} No packages", PACKAGE_ICONS[11]))
                ),
                Some(1) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} 1 package", PACKAGE_ICONS[11]))
                ),
                Some(n) => println!(
                    "{}",
                    calc_whitespace(format!("│ {} {} packages", PACKAGE_ICONS[11], n))
                ),
            },
            Some(&_) | None => match package_count {
                None => (),
                Some(0) => println!("{}", calc_whitespace("│ No packages".to_string())),
                Some(1) => println!("{}", calc_whitespace("│ 1 package".to_string())),
                Some(n) => println!("{}", calc_whitespace(format!("│ {} packages", n))),
            },
        }
    } else {
        match package_count {
            None => (),
            Some(0) => println!("{}", calc_whitespace("│ No packages".to_string())),
            Some(1) => println!("{}", calc_whitespace("│ 1 package".to_string())),
            Some(n) => println!("{}", calc_whitespace(format!("│ {} packages", n))),
        }
    }

    if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => {
                if let Some(song) = song.as_ref() {
                    println!(
                        "{}",
                        calc_whitespace(format!(
                            "│ {} {}",
                            MISC_EMOJIS[5],
                            song.trim_matches('\n')
                        ))
                    );
                }
                println!("╰─────────────────────────────────────────────╯")
            }
            Some("normal") => {
                if let Some(song) = song.as_ref() {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_ICONS[5], song.trim_matches('\n')))
                    );
                }
                println!("╰────────────────────────────────────────────╯")
            }
            Some(&_) | None => {
                if let Some(song) = song.as_ref() {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {}", song.trim_matches('\n')))
                    );
                }
                println!("╰────────────────────────────────────────────╯")
            }
        }
    } else {
        if let Some(song) = song.as_ref() {
            println!(
                "{}",
                calc_whitespace(format!("│ {}", song.trim_matches('\n')))
            );
        }
        println!("╰────────────────────────────────────────────╯")
    }
}
