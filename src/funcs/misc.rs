use {
    crate::util::statics::{CONF, TIME_EMOJIS, TIME_ICONS, WEATHER_EMOJIS, WEATHER_ICONS},
    chrono::{Local, Timelike},
    mpris::PlayerFinder,
    openweathermap::weather,
    substring::Substring,
};

#[tracing::instrument]
pub(crate) fn get_song() -> Option<String> {
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

#[tracing::instrument]
pub(crate) async fn get_weather() -> Option<String> {
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
pub(crate) fn get_datetime() -> Option<String> {
    if !CONF.time.enabled {
        return None;
    }

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
                let index: usize = if dt.hour() >= 12 {
                    (dt.hour() - 12).try_into().unwrap()
                } else {
                    dt.hour().try_into().unwrap()
                };
                TIME_EMOJIS[index.min(11)]
            }
            Some("normal") => {
                let index: usize = if dt.hour() >= 12 {
                    (dt.hour() - 12).try_into().unwrap()
                } else {
                    dt.hour().try_into().unwrap()
                };
                TIME_ICONS[index.min(11)]
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
