use std::collections::VecDeque;
use std::env;

use crate::chip8::{Chip8, Computer, DISPLAY_WIDTH, DISPLAY_HEIGHT};

use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::{
    clear_background, draw_rectangle, is_key_down, is_key_pressed, next_frame, screen_width, Color,
    KeyCode,
};
use macroquad::prelude::{BLACK, BLUE, GREEN, RED, WHITE, YELLOW};

pub const DEFAULT_PIXEL_SIZE: i32 = 20;
const DEFAULT_SPEED_MULTIPLIER: usize = 1;
const DEFAULT_ROM_FILENAME: &str = "./roms/chippi.ch8";
const BUZZ1: &str = "./audio/buzz1.wav";
const BUZZ2: &str = "./audio/buzz2.wav";
const BUZZ3: &str = "./audio/buzz3.wav";
const SOUND_PARAMS: PlaySoundParams = PlaySoundParams {
    looped: false,
    volume: 0.5,
};

pub enum JSEvents{
    ChangeColor(i32),
    SwapRom(String),
    ChangeSpeed(i32),
    ChangeRainbowMode(i32),
}

const ALL_COLORS: [Color; 5] = [GREEN, RED, WHITE, BLUE, YELLOW];

pub struct Program {
    chip: Chip8,
    speed_multiplier: usize,
    color: usize,
    sound: [Sound; 3],
    latch: bool,
    rainbow_mode: bool,
    frame_counter: u8,
}

impl Program {
    pub async fn init(
        rom_filename: String,
        speed_multiplier: usize,
        sound: [Sound; 3],
        rainbow_mode: bool,
    ) -> Program {
        let chip = Chip8::from_rom(Computer::Normal, rom_filename.clone()).await;

        Program {
            chip,
            speed_multiplier,
            sound,
            color: 0,
            rainbow_mode,
            latch: true,
            frame_counter: 0,
        }
    }

    pub async fn run(&mut self, events: &mut Option<VecDeque<JSEvents>>) -> bool {
        while self.process_sys_input() {

            // processing js events
            match events {
                Some(events) => {
                    while let Some(ev) = events.pop_front() {
                        match ev {
                            JSEvents::ChangeColor(new_color) => {
                                self.color = new_color as usize;
                            },
                            JSEvents::SwapRom(rom_filename) => {
                                self.chip = Chip8::from_rom(Computer::Normal, rom_filename).await;
                            },
                            JSEvents::ChangeSpeed(new_speed) => {
                                self.speed_multiplier = new_speed as usize;
                            },
                            JSEvents::ChangeRainbowMode(new_color) => {
                                if self.rainbow_mode {
                                    self.color = new_color as usize;
                                }
                                self.rainbow_mode = !self.rainbow_mode;
                            }
                        }
                    }
                }
                _ => {}
            }

            self.frame_counter = self.frame_counter.wrapping_add(1);
            clear_background(BLACK);
            for _ in 0..self.speed_multiplier {
                self.fill_chip_input();
                self.chip.tick();
                self.process_audio();
            }
            self.draw_chip8_display();
            next_frame().await
        }
        false
    }

    fn increase_color(&mut self) {
        self.color += 1;
        if self.color > ALL_COLORS.len() - 1 {
            self.color = 0;
        }
    }

    fn process_sys_input(&mut self) -> bool {
        if is_key_pressed(KeyCode::Key9) {
            if self.speed_multiplier < 20 {
                self.speed_multiplier += 1;
            }
        }
        if is_key_pressed(KeyCode::Key8) {
            self.speed_multiplier = self.speed_multiplier.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Key0) {
            self.increase_color()
        }
        if is_key_pressed(KeyCode::Escape) {
            return false;
        }
        true
    }

    fn fill_chip_input(&mut self) {
        self.chip.kb[0x0] = is_key_down(KeyCode::X) as u8;
        self.chip.kb[0x1] = is_key_down(KeyCode::Key1) as u8;
        self.chip.kb[0x2] = is_key_down(KeyCode::Key2) as u8;
        self.chip.kb[0x3] = is_key_down(KeyCode::Key3) as u8;
        self.chip.kb[0x4] = is_key_down(KeyCode::Q) as u8;
        self.chip.kb[0x5] = is_key_down(KeyCode::W) as u8;
        self.chip.kb[0x6] = is_key_down(KeyCode::E) as u8;
        self.chip.kb[0x7] = is_key_down(KeyCode::A) as u8;
        self.chip.kb[0x8] = is_key_down(KeyCode::S) as u8;
        self.chip.kb[0x9] = is_key_down(KeyCode::D) as u8;
        self.chip.kb[0xA] = is_key_down(KeyCode::Z) as u8;
        self.chip.kb[0xB] = is_key_down(KeyCode::C) as u8;
        self.chip.kb[0xC] = is_key_down(KeyCode::Key4) as u8;
        self.chip.kb[0xD] = is_key_down(KeyCode::R) as u8;
        self.chip.kb[0xE] = is_key_down(KeyCode::F) as u8;
        self.chip.kb[0xF] = is_key_down(KeyCode::V) as u8;
    }

    fn process_audio(&mut self) {
        if self.latch && self.chip.sreg > 0 {
            match self.chip.sreg {
                10.. => play_sound(self.sound[2], SOUND_PARAMS),
                4..=9 => play_sound(self.sound[1], SOUND_PARAMS),
                _ => play_sound(self.sound[0], SOUND_PARAMS),
            }
            self.latch = false;
        } else if self.chip.sreg == 0 {
            self.latch = true;
        }
    }

    fn get_color(&mut self) -> Color {
        if self.rainbow_mode {
            if self.frame_counter % 10 == 0 {
                self.increase_color();
            }
        }
        ALL_COLORS[self.color]
    }

    fn draw_chip8_display(&mut self) {
        let color = self.get_color();
        for y in 0..DISPLAY_HEIGHT as usize {
            for x in 0..DISPLAY_WIDTH as usize {
                if self.chip.display[y * DISPLAY_WIDTH as usize + x] != 0 {
                    let sw = screen_width() as usize;
                    let pixel_size = sw / DISPLAY_WIDTH as usize;

                    draw_rectangle(
                        (x * pixel_size) as f32,
                        (y * pixel_size) as f32,
                        pixel_size as f32,
                        pixel_size as f32,
                        color,
                    );
                }
            }
        }
    }
}

pub async fn process_env_variables() -> (String, usize, [Sound; 3], bool) {
    let args: Vec<String> = env::args().collect();

    let sound: [Sound; 3] = [
        load_sound(BUZZ1).await.unwrap(),
        load_sound(BUZZ2).await.unwrap(),
        load_sound(BUZZ3).await.unwrap(),
    ];
    
    if args.len() == 1 {
        return (DEFAULT_ROM_FILENAME.to_string(), 1, sound, true);
    }

    let rom_filename = match args.get(1) {
        Some(s) => s.clone(),
        None => DEFAULT_ROM_FILENAME.to_string(),
    };
    let speed_multiplier = match args.get(2) {
        Some(s) => match s.parse::<usize>() {
            Ok(sp) => sp,
            _ => panic!("Speed multiplier not valid"),
        },
        None => DEFAULT_SPEED_MULTIPLIER,
    };
    let rainbow_mode = match args.get(3) {
        Some(_s) => true,
        None => false,
    };

    (rom_filename, speed_multiplier, sound, rainbow_mode)
}