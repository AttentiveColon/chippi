mod lib;

fn get_mq_conf() -> macroquad::prelude::Conf {
    //window config
    macroquad::prelude::Conf {
        window_title: String::from("chippi"),
        window_width: lib::DEFAULT_PIXEL_SIZE * lib::DISPLAY_WIDTH as i32,
        window_height: lib::DEFAULT_PIXEL_SIZE * lib::DISPLAY_HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(get_mq_conf)]
async fn main() {
    let (rom_filename, speed_multiplier, sound, rainbow_mode) = lib::process_env_variables().await;
    let mut program = lib::Program::init(rom_filename, speed_multiplier, sound, rainbow_mode);
    while program.run().await {}
}
