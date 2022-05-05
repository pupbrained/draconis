use {
    crate::statics::CONF,
    std::env,
    substring::Substring,
    sys_info::{linux_os_release, os_release},
    systemstat::{saturating_sub_bytes, Platform, System},
};

#[tracing::instrument]
pub(crate) fn get_release_blocking() -> Option<String> {
    if !CONF.system.release.enabled {
        return None;
    }

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
pub(crate) fn get_kernel_blocking() -> Option<String> {
    if !CONF.system.kernel.enabled {
        return None;
    }

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
pub(crate) fn get_memory() -> Option<String> {
    if !CONF.system.mem_usage.enabled {
        return None;
    }

    match System::new().memory() {
        Ok(mem) => Some(format!(
            "{} Used",
            saturating_sub_bytes(mem.total, mem.free)
        )),
        Err(x) => panic!("Could not get memory because: {}", x),
    }
}

#[tracing::instrument]
pub(crate) fn get_disk_usage() -> Option<String> {
    if !CONF.system.disk_usage.enabled {
        return None;
    }

    match System::new().mount_at("/") {
        Ok(disk) => Some(format!("{} Free", disk.free)),
        Err(x) => panic!("Could not get disk usage because: {}", x),
    }
}

#[tracing::instrument]
pub(crate) fn get_environment() -> Option<String> {
    if !CONF.system.desktop_env.enabled {
        return None;
    }

    Some(
        env::var::<String>(ToString::to_string(&"XDG_CURRENT_DESKTOP")).unwrap_or_else(|_| {
            env::var(&"XDG_SESSION_DESKTOP").unwrap_or_else(|_| "".to_string())
        }),
    )
}
