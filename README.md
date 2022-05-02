# hello-rs
Small terminal welcome program written in rust

![image](https://user-images.githubusercontent.com/33522919/166180428-6f1721c0-01ea-4365-9ae4-4e1409002442.png)

## Requirements
* `pacman-contrib` for pacman

## Important
* This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
* Update checking and package counting will take a long time, and slow down the program by quite a bit. Only use it if you don't mind losing a second or two of time.
  * If you want to skip update checking & package counting, just remove the entire `package_managers` line from the config.

## How to use
* Grab the latest release binary and config files from the releases page
* Copy `example_config.json` to `~/.config/hello-rs/config.json` 
* Change the config to your liking  
* Add the program to your shell's startup
