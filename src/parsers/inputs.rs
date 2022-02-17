pub fn parse_input(v: &String) -> Result<u8, ()> {
    match v.as_str() {
        "None" => Ok(0x00),
        "Exit" => Ok(0x01),
        "ActionA" => Ok(0x02),
        "ActionB" => Ok(0x03),
        "ActionC" => Ok(0x04),
        "ActionH" => Ok(0x05),
        "ActionV" => Ok(0x06),
        "ActionD" => Ok(0x07),
        "MenuL" => Ok(0x08),
        "MenuR" => Ok(0x09),
        "Joy" => Ok(0x0A),
        "Cam" => Ok(0x0B),
        "BumperL" => Ok(0x0C),
        "BumperR" => Ok(0x0D),
        "TriggerL" => Ok(0x0E),
        "TriggerR" => Ok(0x0F),
        "Up" => Ok(0x10),
        "Down" => Ok(0x11),
        "Left" => Ok(0x12),
        "Right" => Ok(0x13),
        "HatUp" => Ok(0x14),
        "HatDown" => Ok(0x15),
        "HatLeft" => Ok(0x16),
        "HatRight" => Ok(0x17),
        "MicUp" => Ok(0x18),
        "MicDown" => Ok(0x19),
        "MicLeft" => Ok(0x1A),
        "MicRight" => Ok(0x1B),
        "PovUp" => Ok(0x1C),
        "PovDown" => Ok(0x1D),
        "PovLeft" => Ok(0x1E),
        "PovRight" => Ok(0x1F),
        "JoyX" => Ok(0x20),
        "JoyY" => Ok(0x21),
        "JoyZ" => Ok(0x22),
        "CamX" => Ok(0x23),
        "CamY" => Ok(0x24),
        "CamZ" => Ok(0x25),
        "Slew" => Ok(0x26),
        "Throttle" => Ok(0x27),
        "ThrottleL" => Ok(0x28),
        "ThrottleR" => Ok(0x29),
        "Volume" => Ok(0x2A),
        "Wheel" => Ok(0x2B),
        "Rudder" => Ok(0x2C),
        "Gas" => Ok(0x2D),
        "Brake" => Ok(0x2E),
        "MicPush" => Ok(0x2F),
        "Trigger" => Ok(0x30),
        "Bumper" => Ok(0x31),
        "ActionL" => Ok(0x32),
        "ActionM" => Ok(0x33),
        "ActionR" => Ok(0x34),
        "Pinky" => Ok(0x35),
        "PinkyForward" => Ok(0x36),
        "PinkyBackward" => Ok(0x37),
        "FlapsUp" => Ok(0x38),
        "FlapsDown" => Ok(0x39),
        "BoatForward" => Ok(0x3A),
        "BoatBackward" => Ok(0x3B),
        "AutopilotPath" => Ok(0x3C),
        "AutopilotAlt" => Ok(0x3D),
        "EngineMotorL" => Ok(0x3E),
        "EngineMotorR" => Ok(0x3F),
        "EngineFuelFlowL" => Ok(0x40),
        "EngineFuelFlowR" => Ok(0x41),
        "EngineIgnitionL" => Ok(0x42),
        "EngineIgnitionR" => Ok(0x43),
        "SpeedbrakeBackward" => Ok(0x44),
        "SpeedbrakeForward" => Ok(0x45),
        "ChinaBackward" => Ok(0x46),
        "ChinaForward" => Ok(0x47),
        "Apu" => Ok(0x48),
        "RadarAltimeter" => Ok(0x49),
        "LandingGearSilence" => Ok(0x4A),
        "Eac" => Ok(0x4B),
        "AutopilotToggle" => Ok(0x4C),
        "ThrottleButton" => Ok(0x4D),
        "MouseX" => Ok(0x4E),
        "MouseY" => Ok(0x4F),
        "Mouse" => Ok(0x50),
        "PaddleLeft" => Ok(0x51),
        "PaddleRight" => Ok(0x52),
        "PinkyLeft" => Ok(0x53),
        "PinkyRight" => Ok(0x54),
        "Context" => Ok(0x55),
        "Dpi" => Ok(0x56),
        "ScrollX" => Ok(0x57),
        "ScrollY" => Ok(0x58),
        "Scroll" => Ok(0x59),
        "TrimUp" => Ok(0x5A),
        "TrimDown" => Ok(0x5B),
        "TrimLeft" => Ok(0x5C),
        "TrimRight" => Ok(0x5D),
        "ActionWheelX" => Ok(0x5E),
        "ActionWheelY" => Ok(0x5F),
        unknown => {
            match unknown.parse::<u8>() {
                Ok(num) => { Ok(num & !0x80) },
                _ => Err(())
            }
        },
    }
}
