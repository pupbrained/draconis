# hello-rs
Small terminal welcome program written in rust

## Important
* If you would like to check updates with pacman, you must have `pacman-contrib` installed.
* If you would like to check updates with portage, you must have `eix` installed.
* This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* If you want to skip update checking, just remove the entire `package_managers` line from the config. If not, it must be an array of strings that consist of "pacman", "apt", "xbps", "portage", and/or "apk".

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking  
* Add the program to your shell's startup
