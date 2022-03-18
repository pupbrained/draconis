# hello-rs
Small (and probably inefficient) terminal welcome program written in rust

## Important
As of right now, this program depends on the `checkupdate` command, meaning this will only work on arch-based systems with the `pacman-contrib` package installed.

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking
  * This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* Add the program to your shell's startup
