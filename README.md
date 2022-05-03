# hello-rs

Small terminal welcome program written in rust

![image](https://user-images.githubusercontent.com/33522919/166400318-8702e241-6cbd-4e79-b517-1e0a2f4a97f0.png)
![image](https://user-images.githubusercontent.com/33522919/166400296-3aaf5238-242f-4ee1-befb-ae4b12725864.png)


## Requirements

- `pacman-contrib` for pacman

## Important

- This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
- Update checking and package counting will take a long time, and slow down the program by quite a bit. Only use it if you don't mind losing a second or two of time.
  - If you want to skip update checking & package counting, just remove the entire `package_managers` line from the config.
- The example config is in JSON5 format for comments only. If you use it for your own config, make sure to change it to a JSON and remove the comments.

## How to use

- Grab the latest release binary and config files from the releases page
- Copy `example_config.json` to `~/.config/hello-rs/config.json`
- Change the config to your liking
- Add the program to your shell's startup
