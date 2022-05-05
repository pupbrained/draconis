use {
    crate::statics::{CONF, PACKAGE_EMOJIS, PACKAGE_ICONS},
    std::process::Stdio,
    tokio::{
        io::{AsyncBufReadExt, BufReader},
        process::{ChildStdout, Command},
    },
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

fn check_update_commmand(command: String) -> Option<(CommandKind, Command)> {
    if !CONF.packages.update_count.enabled {
        return None;
    }
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

#[tracing::instrument]
pub(crate) async fn count_updates() -> Option<String> {
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

fn check_installed_command(command: String) -> Option<(CommandKind, Command)> {
    if !CONF.packages.package_count.enabled {
        return None;
    }
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
pub(crate) async fn get_package_count() -> Option<i32> {
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
