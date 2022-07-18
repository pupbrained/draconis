# Draconis

![image](https://badgen.net/github/release/marsupialgutz/draconis)
![image](https://badgen.net/crates/v/draconis)
![image](https://badgen.net/github/stars/marsupialgutz/draconis)
![image](https://badgen.net/github/commits/marsupialgutz/draconis/main)
![image](https://badgen.net/github/open-prs/marsupialgutz/draconis)
![image](https://badgen.net/github/contributors/marsupialgutz/draconis)


🪐 An out-of-this-world greeter for your terminal

![image](https://user-images.githubusercontent.com/33522919/179458221-d44f7996-d214-46ee-8801-343767ef9295.png)
![image](https://user-images.githubusercontent.com/33522919/179458079-ca750a6d-b1b4-44e4-a721-63e9250780db.png)

## Requirements

- `pacman-contrib` for pacman

## Important

- This program uses the openweathermap API for fetching the weather. You must have an API key from openweathermap if you want to use the weather function.
- Update checking and package counting will take a long time, and slow down the program by quite a bit. This cannot be avoided because of the fact that these checks require external system commands. Only use these options if you don't mind losing a second or two of time every time you run the program.
  - NixOS does not support package update counting.

## Installation
- There are two options for installing draconis: 
  - Run the command `cargo install draconis`
  - Get the binary from the releases and move it into a folder in your PATH

## How to use

- Grab the latest release binary and config files from the releases page
- Copy `example_config.toml` to `~/.config/draconis/config.toml`
- Change the config to your liking
- Add the program to your shell's startup
