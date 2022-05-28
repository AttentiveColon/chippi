use macroquad::prelude::{clear_background, next_frame, BLACK};
use macroquad::audio::{PlaySoundParams};
mod lib;

const DEFAULT_PIXEL_SIZE: i32 = 20;
const SOUND_PARAMS: PlaySoundParams = PlaySoundParams {
    looped: false,
    volume: 0.5,
};

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

#[macroquad::main(get_mq_conf)]
async fn main() {
    let (rom_filename, mut speed_multiplier, sound) = lib::process_env_variables().await;
    let mut chip = lib::Chip8::from_rom(lib::Computer::Normal, rom_filename);
    let mut latch = true;
    loop {
        lib::process_sys_input(&mut speed_multiplier);
        clear_background(BLACK);
        for _ in 0..speed_multiplier {
            lib::fill_chip_input(&mut chip.kb);
            chip.tick();
            lib::process_audio(&mut latch, &chip, sound, SOUND_PARAMS);
        }
        lib::draw_chip8_display(&chip.display);
        next_frame().await
    }
}
