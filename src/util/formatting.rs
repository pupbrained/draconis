use substring::Substring;

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
    let size = ((CONF.util.width - 5) as usize) - text.graphemes(true).count();
    let fs = format!("{}{}", " ".repeat(size), "│");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_whitespace_song(text: String) -> String {
    match ((CONF.util.width).overflowing_sub(3).0 as usize)
        .overflowing_sub(UnicodeWidthStr::width_cjk(text.as_str()))
    {
        (_, false) => {
            let size = ((CONF.util.width).overflowing_sub(3).0 as usize)
                .overflowing_sub(UnicodeWidthStr::width_cjk(text.as_str()))
                .0;
            format!("{}{}", text, format!("{}{}", " ".repeat(size), "│"))
        }
        _ => format!("{}... │", text.substring(0, (CONF.util.width - 9) as usize),),
    }
}

pub(crate) fn calc_with_hostname(text: String) -> String {
    let size = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => ((CONF.util.width + 5) as usize) - text.graphemes(true).count(),
            Some(&_) | None => ((CONF.util.width + 4) as usize) - text.graphemes(true).count(),
        }
    } else {
        ((CONF.util.width + 4) as usize) - text.graphemes(true).count()
    };

    let fs = format!("{}{}", "─".repeat(size), "╮");
    format!("{}{}", text, fs)
}

pub(crate) fn calc_bottom(text: String) -> String {
    let size = if CONF.icons.enabled {
        match CONF.icons.kind.as_deref() {
            Some("emoji") => ((CONF.util.width - 4) as usize) - text.graphemes(true).count(),
            Some(&_) | None => ((CONF.util.width - 5) as usize) - text.graphemes(true).count(),
        }
    } else {
        ((CONF.util.width - 5) as usize) - text.graphemes(true).count()
    };

    let fs = format!("{}{}", "─".repeat(size), "╯");
    format!("{}{}", text, fs)
}
