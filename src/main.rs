extern crate core;

mod program;
mod chip8;

use std::collections::VecDeque;
use program::{DEFAULT_PIXEL_SIZE, JSEvents};
use chip8::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

fn get_mq_conf() -> macroquad::prelude::Conf {
    //window config
    macroquad::prelude::Conf {
        window_title: String::from("chippi"),
        window_width: DEFAULT_PIXEL_SIZE * DISPLAY_WIDTH as i32,
        window_height: DEFAULT_PIXEL_SIZE * DISPLAY_HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

static mut EVENTS: Option<VecDeque<JSEvents>> = None;

#[macroquad::main(get_mq_conf)]
async fn main() {
    let (file, speed, sound, mode) = program::process_env_variables().await;
    let mut program = program::Program::init(file, speed, sound, mode).await;
    unsafe {
        EVENTS = Some(VecDeque::new());
        while program.run(&mut EVENTS).await {}
    }
}

unsafe fn push_event(event: JSEvents) {
    match &mut EVENTS {
        Some(evs) => {
            evs.push_back(event);
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn ev_change_color(color_number: i32) {
    push_event(JSEvents::ChangeColor(color_number));
}

#[no_mangle]
pub unsafe extern "C" fn ev_swap_rom(rom_number: i32) {

    let rom_filename:String = match rom_number {
        0 => {
           "./roms/chippi.ch8".into()
        },
        1 => {
            "./roms/Blinky [Hans Christian Egeberg, 1991].ch8".into()
        },
        2 => {
            "./roms/Breakout (Brix hack) [David Winter, 1997].ch8".into()
        },
        _ => {
            "./roms/Pong (1 player).ch8".into()
        }
    };

    push_event(JSEvents::SwapRom(rom_filename));
}

#[no_mangle]
pub unsafe extern "C" fn ev_change_speed(new_speed: i32) {
    push_event(JSEvents::ChangeSpeed(new_speed));
}

#[no_mangle]
pub unsafe extern "C" fn ev_change_rainbow_mode(color_number: i32) {
    push_event(JSEvents::ChangeRainbowMode(color_number));
}
