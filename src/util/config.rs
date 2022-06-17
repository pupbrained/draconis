use {
    crate::util::conf_structs::Config,
    argparse::{ArgumentParser, Store, StoreTrue},
    std::{env, io::ErrorKind},
};

pub(crate) fn read_config() -> Config {
    let mut path = format!("{}/.config/draconis/config.toml", env::var("HOME").unwrap());
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
        println!("Draconis v{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let content = match std::fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return toml::from_str(
                r#"
                [main]

                [util]

                [greeting]
                enabled = true

                [icons]
                enabled = false

                [time]
                enabled = false

                [weather]
                enabled = false

                [weather.values]

                [system]

                [system.release]
                enabled = false

                [system.kernel]
                enabled = false

                [system.mem_usage]
                enabled = false

                [system.disk_usage]
                enabled = false

                [system.desktop_env]
                enabled = false

                [packages]

                [packages.package_count]
                enabled = false

                [packages.update_count]
                enabled = false

                [song]
                enabled = false
                "#,
            )
            .unwrap()
        }
        Err(e) => panic!("{}", e),
        Ok(content) => match content.as_ref() {
            "" => {
                return toml::from_str(
                    r#"
                [main]

                [util]

                [greeting]
                enabled = true

                [icons]
                enabled = false

                [time]
                enabled = false

                [weather]
                enabled = false

                [weather.values]

                [system]

                [system.release]
                enabled = false

                [system.kernel]
                enabled = false

                [system.mem_usage]
                enabled = false

                [system.disk_usage]
                enabled = false

                [system.desktop_env]
                enabled = false

                [packages]

                [packages.package_count]
                enabled = false

                [packages.update_count]
                enabled = false

                [song]
                enabled = false
                "#,
                )
                .unwrap()
            }
            _ => content,
        },
    };
    toml::from_str(&content).unwrap()
}
