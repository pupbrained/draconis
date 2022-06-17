use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) main: Main,
    pub(crate) util: Util,
    pub(crate) greeting: Greeting,
    pub(crate) icons: Icons,
    pub(crate) time: Time,
    pub(crate) weather: Weather,
    pub(crate) system: System,
    pub(crate) packages: Packages,
    pub(crate) song: Song,
}

#[derive(Deserialize)]
pub(crate) struct Main {
    pub(crate) hostname: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Util {
    pub(crate) width: i32,
}

#[derive(Deserialize)]
pub(crate) struct Greeting {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct Icons {
    pub(crate) enabled: bool,
    pub(crate) kind: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Time {
    pub(crate) enabled: bool,
    pub(crate) kind: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Weather {
    pub(crate) enabled: bool,
    pub(crate) values: WeatherValues,
}

#[derive(Deserialize)]
pub(crate) struct WeatherValues {
    pub(crate) api_key: Option<String>,
    pub(crate) location: Option<String>,
    pub(crate) lang: Option<String>,
    pub(crate) units: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct System {
    pub(crate) release: Release,
    pub(crate) kernel: Kernel,
    pub(crate) mem_usage: MemUsage,
    pub(crate) disk_usage: DiskUsage,
    pub(crate) desktop_env: DesktopEnv,
}

#[derive(Deserialize)]
pub(crate) struct Release {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct Kernel {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct MemUsage {
    pub(crate) enabled: bool,
    pub(crate) free_before_used: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct DiskUsage {
    pub(crate) enabled: bool,
    pub(crate) free_before_used: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct DesktopEnv {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct Packages {
    pub(crate) package_managers: Option<toml::Value>,
    pub(crate) package_count: PackageCount,
    pub(crate) update_count: UpdateCount,
}

#[derive(Deserialize)]
pub(crate) struct PackageCount {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct UpdateCount {
    pub(crate) enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct Song {
    pub(crate) enabled: bool,
    pub(crate) mode: Option<String>,
}
