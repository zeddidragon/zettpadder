Every gamepad remapper for Linux is lacking. Let's change that.

# Installation

1. Clone this repo.
2. 
```
    cargo run
```

On linux you will need x11 dev libraries.

Debian:
```
    sudo apt install libx11-dev
```


# Usage
- Ensure your controller is prepared and plugged in
- Start program
- Run the program and specify your configuration file(s)
  ```
    zettpadder pad-definitions.toml game-mappings.toml
  ```
  Any amount of files can be specified, and they'll be read in sequence

## Mouse emulation
Mouse events fire 120 times per second.

## Layers
Up to 255 layers are supported, including the main one.

# Configuration

Config file is any amount of `.toml` files that contains your definitions.

## Definition

If you fire up the program without arguments and hit buttons, it'll show you output with names for the buttons you press. If disatisfied, you can specify a `[Definitions]` table in your configuration where you rename them. Names can be whatever you want them to be. Running the program with these names will instead output those names and use those for reference in your mapping definitions.

# Features
- Support for every axis and button your device emits, not just the xbox standards
- Map buttons to keyboard output
- Map buttons to mouse output
- Mod keys with ghosting

# Planned features
- Flick stick
- Gyro support

# Not currently planned
- Interface outside of loading a file
