# hello-rs
Small terminal welcome program written in rust

![image](https://user-images.githubusercontent.com/33522919/166089336-0eff3bf4-40ca-4a38-bd38-4a1ceb00b53a.png)

## Requirements
* `pacman-contrib` for pacman
* `eix` for portage
* `iostat` for cpu usage
* `playerctl` for song status

## Important
* This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* If you want to skip update checking & package counting, just remove the entire `package_managers` line from the config.

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking  
* Add the program to your shell's startup
