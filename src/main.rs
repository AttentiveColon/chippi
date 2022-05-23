use macroquad::audio::{load_sound, play_sound, PlaySoundParams};
use macroquad::prelude::*;

use std::env;
use std::io::BufRead;
use std::time::Duration;

use std::thread::sleep;

mod lib;

const DEFAULT_PIXEL_SIZE: i32 = 20;
const DEFAULT_SPEED_MULTIPLIER: usize = 1;
const DEFAULT_ROM_FILENAME: &'static str = "roms/brix.ch8";

fn get_mq_conf() -> macroquad::prelude::Conf {
    //window config
    macroquad::prelude::Conf {
        window_title: String::from("chippi"),
        window_width: DEFAULT_PIXEL_SIZE * lib::DISPLAY_WIDTH as i32,
        window_height: DEFAULT_PIXEL_SIZE * lib::DISPLAY_HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

fn draw_chip8_display(display: &[u8]) {
    const DISPLAYWIDTH: usize = lib::DISPLAY_WIDTH as usize;
    const DISPLAYHEIGHT: usize = lib::DISPLAY_HEIGHT as usize;

    for y in 0..DISPLAYHEIGHT as usize {
        for x in 0..DISPLAYWIDTH as usize {
            if display[y * DISPLAYWIDTH as usize + x] != 0 {
                let sw = screen_width();
                let pixel_size = sw as usize / DISPLAYWIDTH;

                draw_rectangle(
                    (x * pixel_size) as f32,
                    (y * pixel_size) as f32,
                    pixel_size as f32,
                    pixel_size as f32,
                    GREEN,
                );
            }
        }
    }
}

fn fill_input(kb: &mut [u8]) {
    kb[0x0] = is_key_down(KeyCode::X) as u8;
    kb[0x1] = is_key_down(KeyCode::Key1) as u8;
    kb[0x2] = is_key_down(KeyCode::Key2) as u8;
    kb[0x3] = is_key_down(KeyCode::Key3) as u8;
    kb[0x4] = is_key_down(KeyCode::Q) as u8;
    kb[0x5] = is_key_down(KeyCode::W) as u8;
    kb[0x6] = is_key_down(KeyCode::E) as u8;
    kb[0x7] = is_key_down(KeyCode::A) as u8;
    kb[0x8] = is_key_down(KeyCode::S) as u8;
    kb[0x9] = is_key_down(KeyCode::D) as u8;
    kb[0xA] = is_key_down(KeyCode::Z) as u8;
    kb[0xC] = is_key_down(KeyCode::Key4) as u8;
    kb[0xD] = is_key_down(KeyCode::R) as u8;
    kb[0xE] = is_key_down(KeyCode::F) as u8;
    kb[0xF] = is_key_down(KeyCode::V) as u8;
}

use macroquad::audio;

#[macroquad::main(get_mq_conf)]
async fn main() {
    let sound = load_sound("tick.wav").await;
    let sound = sound.unwrap();

    let mut latch = true;

    //TODO(lucypero): consider Chip8::from_rom() instead of new() and load_rom()
    let mut chip = lib::Chip8::new(lib::Computer::Normal);

    //processing args
    let args: Vec<String> = env::args().collect();

    let rom_filename = match args.get(1) {
        Some(s) => s.clone(),
        None => DEFAULT_ROM_FILENAME.to_string(),
    };

    let mut speed_multiplier = match args.get(2) {
        Some(s) => match s.parse::<usize>() {
            Ok(sp) => sp,
            _ => panic!("Speed multiplier not valid."),
        },
        None => DEFAULT_SPEED_MULTIPLIER,
    };

    chip.load_rom(rom_filename).expect("Rom not found");

    loop {
        if is_key_pressed(KeyCode::Key9) {
            speed_multiplier += 1;
        }

        if is_key_pressed(KeyCode::Key8) {
            speed_multiplier -= 1;
        }

        clear_background(BLACK);
        for _ in 0..speed_multiplier {
            fill_input(&mut chip.kb);
            chip.tick();

            if latch && chip.sreg > 0 {
                let sound_params = PlaySoundParams {
                    looped: false,
                    volume: 0.5,
                };
                play_sound(sound, sound_params);
                latch = false;
            } else {
                latch = true;
            }
        }

        draw_chip8_display(&chip.display);
        next_frame().await
    }
}
