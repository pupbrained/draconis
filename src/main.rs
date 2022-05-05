pub mod funcs;
pub mod util;

use {
    crate::{
        funcs::{
            greet::{get_hostname, greeting},
            misc::{get_datetime, get_song, get_weather},
            pkgs::{count_updates, get_package_count},
            system_info::{
                get_disk_usage, get_environment, get_kernel_blocking, get_memory,
                get_release_blocking,
            },
        },
        util::{
            formatting::{calc_whitespace, calc_with_hostname, upper_first},
            statics::{CONF, MISC_EMOJIS, MISC_ICONS, PACKAGE_EMOJIS, PACKAGE_ICONS},
        },
    },
    once_cell::sync::Lazy,
    std::time::Instant,
    tracing_subscriber::{
        fmt::{self, format::FmtSpan},
        prelude::*,
        EnvFilter,
    },
};

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
                if let Some(memory) = memory {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_EMOJIS[2], memory))
                    );
                }
                if let Some(disk) = disk {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_EMOJIS[3], disk))
                    );
                }

                if let Some(environment) = environment {
                    println!(
                        "{}",
                        calc_whitespace(format!(
                            "│ {} {}",
                            MISC_EMOJIS[4],
                            upper_first(environment)
                        ))
                    );
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
                if let Some(memory) = memory {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_ICONS[2], memory))
                    );
                }
                if let Some(disk) = disk {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {} {}", MISC_ICONS[3], disk))
                    );
                }

                if let Some(environment) = environment {
                    println!(
                        "{}",
                        calc_whitespace(format!(
                            "│ {} {}",
                            MISC_ICONS[4],
                            upper_first(environment)
                        ))
                    );
                }
            }
            Some(&_) | None => {
                if let Some(release) = release {
                    println!("{}", calc_whitespace(format!("│ {}", release)));
                }
                if let Some(kernel) = kernel {
                    println!("{}", calc_whitespace(format!("│ {}", kernel)));
                }
                if let Some(memory) = memory {
                    println!("{}", calc_whitespace(format!("│ {}", memory)));
                }
                if let Some(disk) = disk {
                    println!("{}", calc_whitespace(format!("│ {}", disk)));
                }
                if let Some(environment) = environment {
                    println!(
                        "{}",
                        calc_whitespace(format!("│ {}", upper_first(environment)))
                    );
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
        if let Some(memory) = memory {
            println!("{}", calc_whitespace(format!("│ {}", memory)));
        }
        if let Some(disk) = disk {
            println!("{}", calc_whitespace(format!("│ {}", disk)));
        }

        if let Some(environment) = environment {
            println!(
                "{}",
                calc_whitespace(format!("│ {}", upper_first(environment)))
            );
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
