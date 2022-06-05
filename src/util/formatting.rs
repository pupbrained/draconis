use {
    crate::util::statics::CONF, unicode_segmentation::UnicodeSegmentation,
    unicode_width::UnicodeWidthStr,
};

pub(crate) fn upper_first(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn calc_whitespace(text: String) -> String {
    let size = (CONF.util.width - 5) as usize - text.graphemes(true).count();
    let fs = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_whitespace_song(text: String) -> String {
    let size = (CONF.util.width - 3) as usize - UnicodeWidthStr::width_cjk(text.as_str());
    let fs = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_with_hostname(text: String) -> String {
    let size = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => (CONF.util.width + 5) as usize - text.graphemes(true).count(),
            Some(&_) | None => (CONF.util.width + 4) as usize - text.graphemes(true).count(),
        }
    } else {
        (CONF.util.width + 4) as usize - text.graphemes(true).count()
    };

    let fs = format!("{}{}", "─".repeat(size), "╮");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_bottom(text: String) -> String {
    let size = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => (CONF.util.width - 4) as usize - text.graphemes(true).count(),
            Some(&_) | None => (CONF.util.width - 5) as usize - text.graphemes(true).count(),
        }
    } else {
        (CONF.util.width - 5) as usize - text.graphemes(true).count()
    };

    let fs = format!("{}{}", "─".repeat(size), "╯");
    format!("{}{}", text, fs)
}
