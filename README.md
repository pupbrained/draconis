# Draconis

![image](https://badgen.net/github/release/marsupialgutz/draconis)
![image](https://badgen.net/crates/v/draconis)
![image](https://badgen.net/github/stars/marsupialgutz/draconis)
![image](https://badgen.net/github/commits/marsupialgutz/draconis/main)
![image](https://badgen.net/github/open-prs/marsupialgutz/draconis)
![image](https://badgen.net/github/contributors/marsupialgutz/draconis)


🪐 An out-of-this-world greeter for your terminal

![image](https://user-images.githubusercontent.com/33522919/166400318-8702e241-6cbd-4e79-b517-1e0a2f4a97f0.png)
![image](https://user-images.githubusercontent.com/33522919/166400296-3aaf5238-242f-4ee1-befb-ae4b12725864.png)

## Requirements

- `pacman-contrib` for pacman

## Important

- This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap for it to work.
- Update checking and package counting will take a long time, and slow down the program by quite a bit. This cannot be avoided because of the fact that these checks require external system commands. Only use these options if you don't mind losing a second or two of time every time you run the program.
- The config format has recently **changed from JSON to TOML.** Make sure you're using the TOML file properly from now on.

## How to use

- Grab the latest release binary and config files from the releases page
- Copy `example_config.toml` to `~/.config/draconis/config.toml`
- Change the config to your liking
- Add the program to your shell's startup
