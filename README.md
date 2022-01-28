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
- Press every button on it
- Program will write out to `output.toml` with code -> key definitions. You may use and modify this as a basis for your templates. Key names can be whatever you want them to be, but they should all be named different things.
- See `./examples` folder for examples on how to use this.
- Run the program and specify your configuration file(s)

  ```
    zettpadder pad-definitions.toml game-mappings.toml
  ```
  Any amount of files can be specified, and they'll be read in sequence

TODO

# Planned features
- Support for every axis and button your device emits, not just the xbox standards
- Map buttons to keyboard output and mouse output
- Mod keys with ghosting
- Flick stick

# Not currently planned
- Interface outside of loading a file
