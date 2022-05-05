use {crate::util::statics::CONF, unicode_segmentation::UnicodeSegmentation};

pub(crate) fn upper_first(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn calc_whitespace(text: String) -> String {
    let size = 45 - text.graphemes(true).count();
    let fs = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_with_hostname(text: String) -> String {
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
