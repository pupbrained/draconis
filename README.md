# hello-rs
Small terminal welcome program written in rust

![image](https://user-images.githubusercontent.com/33522919/159401366-157bc32d-bf81-4d31-94cd-161abde0c5e5.png)

## Important
* If you would like to check updates with pacman, you must have `pacman-contrib` installed.
* If you would like to check updates with portage, you must have `eix` installed.
* This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* If you want to skip update checking & package counting, just remove the entire `package_managers` line from the config. If not, it must be an array of strings that consist of "pacman", "apt", "xbps", "portage", and/or "apk".
* If you want to use the song status, you must be using `playerctl`. Otherwise, just set the `song` boolean in the config to `false`.

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking  
* Add the program to your shell's startup
