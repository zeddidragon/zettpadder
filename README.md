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
    sudo apt install build-essential libx11-dev libxi-dev libxtst-dev
```


# Usage
- Ensure your controller is prepared and plugged in
- Start program
- Run the program and specify your configuration file(s)
  ```
    zettpadder some-definitions.zett game-definitions.zett
  ```
  Any amount of files can be specified, and they'll be read in sequence

## Layers
Up to 255 layers are supported, including the main one.

# Configuration


Config file is any amount of `.zett` files that contains your definitions.

## Input Set Shortcuts
```
  <inputset> <bind1> <bind2> <bind3> <bind4>
  <inputset> <shortcut>
```

Possible input sets:
- JoyXY
- CamXY
- MouseXY
- ActionWheelXY
- DpadXY
- PovXY
- HatXY
- MicXY

Possible output shortcuts:
- WASD (A D W S)
- IJKL (J L I K)
- Arrows (Left Right Up Down)
- Mouse (X, Y)
- Flick (FlickX, FlickY)

## Definition

If you fire up the program without arguments and hit buttons, it'll show you output with names for the buttons you press. You can use those for reference in your mapping definitions. You can toggle echo mode with the command `echo <on|off>`.

# Features
- Support for every axis and button your device emits, not just the xbox standards.
- Map buttons to keyboard output.
- Map buttons to mouse output.
- Mod keys with ghosting.
- Flick stick.

# Planned features
- Radial functions.
- Gyro support.
- PS4 touchpad suppport.
- Game overlay to support radial menus etc.

# Not currently planned
- Interface outside of input commands.
- Support for mapping multiple controllers simultaneously. Just run the program twice.

## Addendum: Disabling Flydigi's mouse click on right grip button
1. Identify ID of the device:
```
    xinput list | grep Flydigi | grep pointer
```
2. Disable button
```
    xinput set-button-map <id> 0
```

