# Draconis

![image](https://badgen.net/github/release/marsupialgutz/draconis)
![image](https://badgen.net/crates/v/draconis)
![image](https://badgen.net/github/stars/marsupialgutz/draconis)
![image](https://badgen.net/github/commits/marsupialgutz/draconis/main)
![image](https://badgen.net/github/open-prs/marsupialgutz/draconis)
![image](https://badgen.net/github/contributors/marsupialgutz/draconis)


🪐 An out-of-this-world greeter for your terminal

![image](https://user-images.githubusercontent.com/33522919/170403598-a04f7859-6130-4887-b291-77ef957a3034.png)
![image](https://user-images.githubusercontent.com/33522919/170403547-eb078215-10b7-4c77-8cad-fde0b011946f.png)

## Requirements

- `pacman-contrib` for pacman

## Important

- This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap if you want to use the weather function.
- Update checking and package counting will take a long time, and slow down the program by quite a bit. This cannot be avoided because of the fact that these checks require external system commands. Only use these options if you don't mind losing a second or two of time every time you run the program.
  - NixOS does not support package update counting.

## How to use

- Grab the latest release binary and config files from the releases page
- Copy `example_config.toml` to `~/.config/draconis/config.toml`
- Change the config to your liking
- Add the program to your shell's startup
