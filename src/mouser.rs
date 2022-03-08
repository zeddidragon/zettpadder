use std::time::Duration;
use std::f64::consts::{PI, TAU};
use crossbeam_channel::{tick, Sender, Receiver};
use crate::coords::{Coords};
use crate::smoothing::{Smoothing};
use crate::zettpadder::{ZpMsg};

const FPS: u64 = 60;  // Default loop rate
const FLICK_FACTOR: f64 = 1280.0;  // How much one radian moves the mouse
const FLICK_DEADZONE: f64 = 0.9; // Deadzone to engage flick
const FLICK_TIME: Duration = Duration::from_millis(100); // Duration of a flick

#[derive(Debug, Copy, Clone)]
pub enum MousePriority {
    Flick,
    Motion,
    Mixed,
}

fn send(sender: &Sender<ZpMsg>, msg: ZpMsg) {
    match sender.send(msg) {
        Err(err) => {
            println!("Unable to send to zettpadder: {:?}\n{:?}", msg, err);
        },
        _ => {},
    };
}

#[inline]
fn modulo(v: f64, k: f64) -> f64 {
    v - (v / k).floor() * k
}

#[derive(Debug, Copy, Clone)]
pub enum MouserMsg {
    SetFps(u64), // Cycle rate of main loop
    SetFlickFactor(f64), // Mouse motion of one radian
    SetFlickTime(u64), // Duration of a flick
    SetFlickDeadzone(f64), // Deadzone of stick before initiating flick
    GetFlickCalibration(f64), // Display data to help calibrate flick factor
    SetMousePriority(MousePriority), // Wether to prioritize mouse or flick input

    MouseX(f64), // Assign mouse X axis
    MouseY(f64), // Assign mouse Y axis
    FlickX(f64), // Assign flick X axis
    FlickY(f64), // Assign flick Y axis
}

pub fn run(sender: Sender<ZpMsg>, receiver: Receiver<MouserMsg>) {
    let mut tick_time = Duration::from_nanos(1_000_000_000 / FPS);
    let mut ticker = tick(tick_time);

    let mut mover = Coords::new();
    let mut motion = Coords::new();
    let mut mouse_priority = MousePriority::Mixed;

    let mut flicker = Coords::new();
    let mut prev_flicker;
    let mut flick_deadzone = FLICK_DEADZONE;
    let mut flick_smoother = Smoothing::new();
    let mut total_flick_steering: f64 = 0.0;
    let mut flick_time = FLICK_TIME;
    let mut flick_remaining = 0;
    let mut flick_tick = 0.0;
    let mut flick_factor = FLICK_FACTOR;

    loop {
        prev_flicker = flicker;

        while let Ok(msg) = receiver.try_recv() {
            use MouserMsg::*;
            match msg {
                SetFps(v) => {
                    tick_time = Duration::from_nanos(1_000_000_000 / v);
                    ticker = tick(tick_time);
                },
                SetFlickFactor(v) => { flick_factor = v; },
                SetFlickTime(v) => { flick_time = Duration::from_millis(v); },
                SetFlickDeadzone(v) => { flick_deadzone = v / 100.0; },

                GetFlickCalibration(v) => {
                    let steering = total_flick_steering.abs();
                    println!("Recommended flickfactor: {}", steering / TAU / v);
                },
                SetMousePriority(v) => { mouse_priority = v; },

                MouseX(v) => { mover.x = v; },
                MouseY(v) => { mover.y = v; },
                FlickX(v) => { flicker.x = v; },
                FlickY(v) => { flicker.y = v; },
            }
        }


        ticker.recv().unwrap();

        let mut can_flick = true;
        // Old school moving
        if mover.len() > 0.0 {
            motion = mover;
        } else {
            motion *= 0.0;
        }

        match mouse_priority {
            MousePriority::Motion => {
                if mover.manhattan() > 0.0 {
                    can_flick = false;
                }
            },
            MousePriority::Flick => {
                if flicker.manhattan() > 0.0 {
                   motion *= 0.0;
                }
            },
            MousePriority::Mixed => {},
        }

        // Flick sticking
        if can_flick && flicker.len() >= flick_deadzone {
            if prev_flicker.len() < flick_deadzone {
                // Starting a flick
                let angle = flicker.angle();
                flick_remaining = (
                    flick_time.as_nanos()
                    / tick_time.as_nanos()).max(1) as u64;
                flick_tick = flick_factor * angle / (flick_remaining as f64);
                flick_smoother.clear();
                total_flick_steering = 0.0;

            } else {
                // Steering
                let angle = flicker.angle();
                let prev_angle = prev_flicker.angle();
                let steering = flick_factor * flick_smoother.tier_smooth(
                    modulo(angle - prev_angle + PI, TAU) - PI);
                total_flick_steering += steering;
                motion.x += steering;
            }
        }

        if flick_remaining > 0 {
            flick_remaining -= 1;
            motion.x += flick_tick;
        }

        // Apply all motion in the tick
        if motion.manhattan() > 0.0 {
            let event = rdev::EventType::MouseMoveRelative {
                delta_x: motion.x,
                delta_y: motion.y,
            };
            send(&sender, ZpMsg::Output(event));
        }
    }
}
