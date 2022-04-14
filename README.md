# hello-rs
Small terminal welcome program written in rust

![image](https://user-images.githubusercontent.com/33522919/163493454-54e5db2b-926a-484d-ae91-052cf5a13a0d.png)

## Important
* If you would like to check updates with pacman, you must have `pacman-contrib` installed.
* If you would like to check updates with portage, you must have `eix` installed.
* This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* If you want to skip update checking & package counting, just remove the entire `package_managers` line from the config.
* Using the song status requires `playerctl`.

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking  
* Add the program to your shell's startup
