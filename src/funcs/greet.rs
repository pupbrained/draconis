use {
    crate::util::statics::{CONF, GREET_EMOJIS, GREET_ICONS},
    chrono::{Local, Timelike},
    sys_info::hostname,
    whoami::{realname, username},
};

#[tracing::instrument]
pub(crate) fn get_hostname() -> Option<String> {
    match &CONF.main.hostname {
        Some(hostname) => Some(hostname.to_string()),
        None => Some(format!("{}@{}", username(), hostname().ok()?)),
    }
}

#[tracing::instrument]
pub(crate) fn greeting() -> Option<String> {
    if !CONF.greeting.enabled {
        return None;
    }

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
