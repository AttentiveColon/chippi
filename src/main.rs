mod program;
mod chip8;

use program::DEFAULT_PIXEL_SIZE;
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

#[macroquad::main(get_mq_conf)]
async fn main() {
    let (file, speed, sound, mode) = program::process_env_variables().await;
    let mut program = program::Program::init(file, speed, sound, mode);
    while program.run().await {}
}
