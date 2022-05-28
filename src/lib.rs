#![allow(dead_code, non_snake_case)]

use rand::Rng;
use std::env;

use macroquad::audio::{Sound, PlaySoundParams, load_sound, play_sound};
use macroquad::prelude::{is_key_pressed, is_key_down, KeyCode, draw_rectangle, screen_width, clear_background, next_frame};
use macroquad::prelude::{GREEN, BLACK};

pub const DEFAULT_PIXEL_SIZE: i32 = 20;
pub const DEFAULT_SPEED_MULTIPLIER: usize = 1;
pub const DEFAULT_ROM_FILENAME: &'static str = "roms/brix.ch8";
pub const AUDIO_FILE: &str = "tick.wav";
pub const SOUND_PARAMS: PlaySoundParams = PlaySoundParams {
    looped: false,
    volume: 0.5,
};

pub struct Program {
    chip: Chip8,
    speed_multiplier: usize,
    sound: Sound,
    rom_filename: String,
    
}

impl Program {
    pub fn init(rom_filename: String, speed_multiplier: usize, sound: Sound) -> Program {
        let chip = Chip8::from_rom(Computer::Normal, rom_filename.clone());

        Program {
            chip: chip,
            speed_multiplier: speed_multiplier,
            sound: sound,
            rom_filename: rom_filename,
        }
    }

    pub async fn run(&mut self) -> bool {
        let mut latch = true;
        loop {
            if !process_sys_input(&mut self.speed_multiplier) {
                break;
            }
            clear_background(BLACK);
            for _ in 0..self.speed_multiplier {
                fill_chip_input(&mut self.chip.kb);
                self.chip.tick();
                process_audio(&mut latch, &self.chip, self.sound, SOUND_PARAMS);
            }
            draw_chip8_display(&self.chip.display);
            next_frame().await
        }
        false
    }
}


pub async fn process_env_variables() -> (String, usize, Sound) {
    let args: Vec<String> = env::args().collect();
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
    let sound = load_sound(AUDIO_FILE).await.unwrap();

    (rom_filename, speed_multiplier, sound)
}

pub fn process_audio(latch: &mut bool, chip: &Chip8, sound: Sound, sound_params: PlaySoundParams) {
    if *latch && chip.sreg > 0  {
        play_sound(sound, sound_params);
        *latch = false;
    } else if chip.sreg == 0 {
        *latch = true;
    }
}

pub fn process_sys_input(speed_multiplier: &mut usize) -> bool {
    if is_key_pressed(KeyCode::Key9) {
        *speed_multiplier += 1;
    }
    if is_key_pressed(KeyCode::Key8) {
        *speed_multiplier -= 1;
    }
    if is_key_pressed(KeyCode::Escape) {
        return false;
    }
    true
}

pub fn fill_chip_input(kb: &mut [u8]) {
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
    kb[0xB] = is_key_down(KeyCode::C) as u8;
    kb[0xC] = is_key_down(KeyCode::Key4) as u8;
    kb[0xD] = is_key_down(KeyCode::R) as u8;
    kb[0xE] = is_key_down(KeyCode::F) as u8;
    kb[0xF] = is_key_down(KeyCode::V) as u8;
}

pub fn draw_chip8_display(display: &[u8]) {
    const DISPLAYWIDTH: usize = DISPLAY_WIDTH as usize;
    const DISPLAYHEIGHT: usize = DISPLAY_HEIGHT as usize;

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

#[derive(Debug)]
pub enum Chip8Error {
    FileNotFound,
}

impl std::fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Chip8Error::FileNotFound => "File not found",
            }
        )
    }
}

pub enum Computer {
    Normal,
    Eti,
}

pub enum Nibble {
    Second,
    Third,
    Last,
    Byte,
    Address,
}

//0xabcd
fn g_nib(nib: Nibble, addr: u16) -> u16 {
    match nib {
        Nibble::Second => (addr >> 8) & 0xF,
        Nibble::Third => (addr >> 4) & 0xF,
        Nibble::Last => addr & 0xF,
        Nibble::Byte => addr & 0xFF,
        Nibble::Address => addr & 0xFFF,
    }
}

const PROGRAM_START_LOCATION: usize = 0x200;
const ETI_PROGRAM_START_LOCATION: usize = 0x600;
const TEXT_MEMORY_START: usize = 0x000;
pub const DISPLAY_WIDTH: u8 = 64;
pub const DISPLAY_HEIGHT: u8 = 32;

#[rustfmt::skip]
const TEXT_ARRAY: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, //2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, //3
    0x90, 0x90, 0xf0, 0x10, 0x10, //4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, //5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, //6
    0xf0, 0x10, 0x20, 0x40, 0x40, //7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, //8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, //9
    0xf0, 0x90, 0xf0, 0x90, 0x90, //A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, //B
    0xf0, 0x80, 0x80, 0x80, 0xf0, //C
    0xe0, 0x90, 0x90, 0x90, 0xe0, //D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, //E
    0xf0, 0x80, 0xf0, 0x80, 0x80, //F
];

fn load_text(offset: usize) -> [u8; 4096] {
    let mut ram = [0x0; 4096];

    for i in offset..TEXT_ARRAY.len() {
        ram[i] = TEXT_ARRAY[i - offset];
    }
    ram
}

fn get_nibble(n: u8, addr: u16) -> u16 {
    match n {
        0 => (addr & 0xF000) >> 12,
        1 => (addr & 0x0F00) >> 8,
        2 => (addr & 0x00F0) >> 4,
        _ => addr & 0x000F,
    }
}

pub struct Chip8 {
    pub ram: [u8; 4096],
    regs: [u8; 16],   //general purpose registers. but the last one is reserved
    ireg: u16,        //i reg. used to store memory addresses
    dreg: u8,         // delay timer register
    pub sreg: u8,     // sound timer register
    pc: u16,          // program counter
    sp: u8,           // stack pointer (index to stack)
    stack: [u16; 16], // stack. array of pointers
    pub kb: [u8; 16], // the keyboard
    pub display: [u8; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
    rng: rand::rngs::ThreadRng,
}

impl Chip8 {
    pub fn new(comp: Computer) -> Chip8 {
        Chip8 {
            ram: load_text(TEXT_MEMORY_START),
            regs: [0x0; 16],
            ireg: 0x00,
            dreg: 0x0,
            sreg: 0x0,
            sp: 0x0,
            stack: [0x00; 16],
            kb: [0x0; 16],
            display: [0; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
            pc: match comp {
                Computer::Normal => PROGRAM_START_LOCATION,
                Computer::Eti => ETI_PROGRAM_START_LOCATION,
            } as u16,
            rng: rand::thread_rng(),
        }
    }

    pub fn from_rom(comp: Computer, rom: String) -> Chip8 {
        let mut chip8 = Chip8 { 
            ram: load_text(TEXT_MEMORY_START), 
            regs: [0x0; 16], 
            ireg: 0x00, 
            dreg: 0x00, 
            sreg: 0x0, 
            pc: match comp { 
                Computer::Normal => PROGRAM_START_LOCATION, 
                Computer::Eti => ETI_PROGRAM_START_LOCATION
            } as u16, 
            sp: 0x0, 
            stack: [0x00; 16], 
            kb: [0x0; 16], 
            display: [0; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize], 
            rng: rand::thread_rng() 
        };
        chip8.load_rom(rom).expect("Couldn't Load Rom");
        chip8
    }

    pub fn load_rom(&mut self, path: String) -> Result<(), Chip8Error> {
        let the_file: Vec<u8> = std::fs::read(path).map_err(|_| Chip8Error::FileNotFound)?;

        for (dst, src) in self
            .ram
            .iter_mut()
            .skip(PROGRAM_START_LOCATION)
            .zip(&the_file)
        {
            *dst = *src;
        }

        Ok(())
    }

    //executes the instruction on pc and changes all the state
    //ram[pc] + ram[pc + 1]
    pub fn tick(&mut self) {
        let instruction =
            ((self.ram[self.pc as usize] as u16) << 8) | self.ram[self.pc as usize + 1] as u16;

        if self.dreg > 0 {
            self.dreg = self.dreg.saturating_sub(1);
        }
        if self.sreg > 0 {
            self.sreg = self.sreg.saturating_sub(1);
        }

        //get the first nibble and pattern match it
        match get_nibble(0, instruction) {
            0x0 => {
                //this can be 1 of 3 instr
                match instruction {
                    //CLS
                    0x00E0 => self.CLS(),
                    //RET
                    0x00EE => self.RET(),
                    //SYS
                    _ => self.SYS(g_nib(Nibble::Address, instruction)),
                }
            }
            0x1 => self.JP(g_nib(Nibble::Address, instruction)),
            0x2 => self.CALL(g_nib(Nibble::Address, instruction)),
            0x3 => self.SE(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Byte, instruction) as u8,
            ),
            0x4 => self.SNE(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Byte, instruction) as u8,
            ),
            0x5 => self.SER(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Third, instruction) as u8,
            ),
            0x6 => self.LD(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Byte, instruction) as u8,
            ),
            0x7 => self.ADD(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Byte, instruction) as u8,
            ),
            0x8 => match g_nib(Nibble::Last, instruction) {
                0x0 => self.LDR(
                    g_nib(Nibble::Second, instruction) as u8 as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x1 => self.OR(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x2 => self.AND(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x3 => self.XOR(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x4 => self.ADDR(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x5 => self.SUB(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x6 => self.SHR(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0x7 => self.SUBN(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                0xE => self.SHL(
                    g_nib(Nibble::Second, instruction) as u8,
                    g_nib(Nibble::Third, instruction) as u8,
                ),
                _ => panic!(),
            },
            0x9 => self.SNER(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Third, instruction) as u8,
            ),
            0xA => self.LDI(g_nib(Nibble::Address, instruction)),
            0xB => self.JPO(g_nib(Nibble::Address, instruction)),
            0xC => self.RND(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Byte, instruction) as u8,
            ),
            0xD => self.DRW(
                g_nib(Nibble::Second, instruction) as u8,
                g_nib(Nibble::Third, instruction) as u8,
                g_nib(Nibble::Last, instruction) as u8,
            ),
            0xE => match g_nib(Nibble::Byte, instruction) {
                0x9E => self.SKPK(g_nib(Nibble::Second, instruction) as u8),
                0xA1 => self.SKNPK(g_nib(Nibble::Second, instruction) as u8),
                _ => panic!(),
            },
            0xF => match g_nib(Nibble::Byte, instruction) {
                0x07 => self.LDT(g_nib(Nibble::Second, instruction) as u8),
                0x0A => self.LDK(g_nib(Nibble::Second, instruction) as u8),
                0x15 => self.LDD(g_nib(Nibble::Second, instruction) as u8),
                0x18 => self.LDS(g_nib(Nibble::Second, instruction) as u8),
                0x1E => self.ADDI(g_nib(Nibble::Second, instruction) as u8),
                0x29 => self.LDF(g_nib(Nibble::Second, instruction) as u8),
                0x33 => self.LDB(g_nib(Nibble::Second, instruction) as u8),
                0x55 => self.LDIX(g_nib(Nibble::Second, instruction) as u8),
                0x65 => self.LDRX(g_nib(Nibble::Second, instruction) as u8),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    /// 0nnn - SYS addr
    /// Jump to a machine code routine at nnn.
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented.
    /// It is ignored by modern interpreters.
    fn SYS(&mut self, _addr: u16) {
        //not implemented
        self.pc += 2;
    }

    /// 0e00 - CLS
    /// Clear the display.
    fn CLS(&mut self) {
        self.display[..].fill(0x0);
        self.pc += 2;
    }

    /// 00ee - RET
    /// Return from a subroutine
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn RET(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
        self.pc += 2;
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    pub fn JP(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack.
    /// The PC is then set to nnn.
    fn CALL(&mut self, addr: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr;
    }

    /// 3xkk - SE Vx, Byte
    /// Skip the next instruction if Vx = kk.
    /// The interpreter compare register Vx to kk, and if they are equal, increments the program counter by 2.
    fn SE(&mut self, x: u8, kk: u8) {
        if self.regs[x as usize] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// 4xkk - SNE Vx, Byte
    /// Skip next instruction if Vx != kk.
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn SNE(&mut self, x: u8, kk: u8) {
        if self.regs[x as usize] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// 5xy0 - SER Vx, Vy
    /// Skip next instruction if Vx = Vy.
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn SER(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] == self.regs[y as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// 6xkk - LD Vx, Byte
    /// Set Vx = kk
    /// The interpreter puts the value kk into register Vx.
    fn LD(&mut self, x: u8, kk: u8) {
        self.regs[x as usize] = kk;
        self.pc += 2;
    }

    /// 7xkk = ADD Vx, Byte
    /// Set Vx = Vx + kk
    /// Adds the value kk to the value of register Vx, then stores the result in Vx
    fn ADD(&mut self, x: u8, kk: u8) {
        //self.regs[x as usize] += kk;
        self.regs[x as usize] = self.regs[x as usize].wrapping_add(kk);
        self.pc += 2;
    }

    /// 8xy0 - LDR Vx, Vy
    /// Set Vx = Vy
    /// Stores the value of register Vy in register Vx.
    fn LDR(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[y as usize];
        self.pc += 2;
    }

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy
    /// Performs a bitwise Or on the values of Vx and Vy, then stores the result in Vx.
    fn OR(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[x as usize] | self.regs[y as usize];
        self.pc += 2;
    }

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    fn AND(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[x as usize] & self.regs[y as usize];
        self.pc += 2;
    }

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    fn XOR(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[x as usize] ^ self.regs[y as usize];
        self.pc += 2;
    }

    /// 8xy4 = ADDR Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits VF is set to 1, Otherwise 0.
    /// Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn ADDR(&mut self, x: u8, y: u8) {
        let result: u16 = self.regs[x as usize] as u16 + self.regs[y as usize] as u16;
        if result > u8::MAX as u16 {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }

        self.regs[x as usize] = (result & 0x00FF) as u8;
        self.pc += 2;
    }

    /// 8xy5 = SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn SUB(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] > self.regs[y as usize] {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }

        self.regs[x as usize] = self.regs[x as usize].wrapping_sub(self.regs[y as usize]);
        self.pc += 2;
    }

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx SHR 1
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn SHR(&mut self, x: u8, _y: u8) {
        if self.regs[x as usize] & 0x1 == 0x1 {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }

        self.regs[x as usize] = self.regs[x as usize] >> 1;
        self.pc += 2;
    }

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the result is stored in Vx.
    fn SUBN(&mut self, x: u8, y: u8) {
        if self.regs[y as usize] > self.regs[x as usize] {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }

        self.regs[x as usize] = self.regs[y as usize].wrapping_sub(self.regs[x as usize]);
        self.pc += 2;
    }

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx SHL 1
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is multiplied by 2.
    fn SHL(&mut self, x: u8, _y: u8) {
        if self.regs[x as usize] & 0x80 != 0 {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }

        self.regs[x as usize] = self.regs[x as usize] << 1;
        self.pc += 2;
    }

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction is Vx != Vy
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn SNER(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] != self.regs[y as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Annn - LDI I, addr
    /// Set I = nnn.
    /// The value of register I is set to nnn.
    fn LDI(&mut self, addr: u16) {
        self.ireg = addr;
        self.pc += 2;
    }

    /// Bnnn JPO V0, addr
    /// Jump to location nnn + V0
    /// The program counter is set to nnn plus the value of V0
    fn JPO(&mut self, addr: u16) {
        self.pc = addr + self.regs[0] as u16;
    }

    /// Cxkk - RND Vx, Byte
    /// Set Vx = random byte AND kk
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    /// The results are stored in Vx.
    fn RND(&mut self, x: u8, kk: u8) {
        self.regs[x as usize] = self.rng.gen::<u8>() & kk;
        self.pc += 2;
    }

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed
    /// as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels
    /// to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the
    /// coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more
    /// information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    fn DRW(&mut self, x: u8, y: u8, n: u8) {
        self.regs[0xF] = 0;
        for i in 0..n as u32 {
            //the sprite row

            let byte_index = self.ireg as usize + i as usize;
            let mut current_byte = self.ram[byte_index];

            for j in 0..8 as u32 {
                let position = ((self.regs[y as usize] as u32 + i) % DISPLAY_HEIGHT as u32)
                    * DISPLAY_WIDTH as u32
                    + (self.regs[x as usize] as u32 + j) % DISPLAY_WIDTH as u32;

                let current_bit = (current_byte & 0x80) >> 7;

                if self.display[position as usize] != 0 && current_bit != 0 {
                    self.regs[0xF] = 1;
                } //else {
                  // self.regs[0xF] = 0;
                  // }

                self.display[position as usize] ^= current_bit;
                current_byte <<= 1;
            }
        }

        self.pc += 2;
    }

    /// Ex9E - SKPK Vx
    /// Skip next instruction if key with the value of Vx is pressed
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    fn SKPK(&mut self, x: u8) {
        if self.kb[self.regs[x as usize] as usize] != 0 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// ExA1 - SKNPK Vx
    /// Skip the next instruction if key with the value of Vx is not pressed.
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    fn SKNPK(&mut self, x: u8) {
        if self.kb[self.regs[x as usize] as usize] == 0 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Fx07 - LDT Vx, DT
    /// Set Vx = delay timer value
    /// The value of DT is placed into Vx.
    fn LDT(&mut self, x: u8) {
        self.regs[x as usize] = self.dreg;
        self.pc += 2;
    }

    /// Fx0A - LDK Vx, K
    /// Wait for a key press, store the value of the key in Vx
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn LDK(&mut self, x: u8) {
        for i in self.kb {
            if i != 0 {
                self.regs[x as usize] = i;
                self.pc += 2;
                break;
            }
        }
    }

    /// Fx15 - LDD DT, Vx
    /// Set delay timer = Vx
    /// DT is set equal to value of Vx.
    fn LDD(&mut self, x: u8) {
        self.dreg = self.regs[x as usize];
        self.pc += 2;
    }

    /// Fx18 - LDS ST, Vx
    /// Set sound timer = Vx
    /// ST is set equal to the value of Vx.
    fn LDS(&mut self, x: u8) {
        self.sreg = self.regs[x as usize];
        self.pc += 2;
    }

    /// Fx1E - ADDI I, Vx
    /// Set I = I + Vx
    /// The values of I and Vx are added, and the results are stored in I.
    fn ADDI(&mut self, x: u8) {
        self.ireg += self.regs[x as usize] as u16;
        self.pc += 2;
    }

    /// Fx29 - LDF F, Vx
    /// Set I = Location of sprite for digit Vx.
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    fn LDF(&mut self, x: u8) {
        self.ireg = self.regs[x as usize] as u16 * 5 + TEXT_MEMORY_START as u16;
        self.pc += 2;
    }

    /// Fx33 - LDB B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2
    /// The integer takes the decimal value of Vx, and places the hundreds digit in memory at location I, the tens digit in location I+1,
    /// and the ones digits at location I+2.
    fn LDB(&mut self, x: u8) {
        let mut value = self.regs[x as usize];
        for i in (0..=2).rev() {
            self.ram[(self.ireg + i) as usize] = value % 10;
            value /= 10;
        }
        self.pc += 2;
    }

    /// Fx55 - LDIX [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    /// The interpreter copies the values of registers v0 through Vx into memory, starting at the address in I.
    fn LDIX(&mut self, x: u8) {
        for (i, val) in self.regs.into_iter().take((x + 1) as usize).enumerate() {
            self.ram[self.ireg as usize + i] = val;
        }
        self.pc += 2;
    }

    /// Fx65 - LDRX Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn LDRX(&mut self, x: u8) {
        for (i, val) in self
            .ram
            .into_iter()
            .skip(self.ireg as usize)
            .take((x + 1) as usize)
            .enumerate()
        {
            self.regs[i] = val;
        }
        self.pc += 2;
    }
}

#[test]
fn tests() {
    let addr: u16 = 0xABCD;

    assert_eq!(get_nibble(0, addr), 0xA);
    assert_eq!(get_nibble(1, addr), 0xB);
    assert_eq!(get_nibble(2, addr), 0xC);
    assert_eq!(get_nibble(3, addr), 0xD);
}

// #[test]
// fn test_get_nibbles() {
//     let addr: u16 = 0xABCD;
//     assert_eq!(get_nibbles(0, 1, addr), 0xAB);
//     assert_eq!(get_nibbles(1, 2, addr), 0xBC);
//     assert_eq!(get_nibbles(1, 1, addr), 0xB);
//     assert_eq!(get_nibbles(1, 3, addr), 0xBCD);
// }

#[test]
fn test_g_nibs() {
    let addr: u16 = 0xABCD;
    assert_eq!(g_nib(Nibble::Second, addr), 0xB);
    assert_eq!(g_nib(Nibble::Third, addr), 0xC);
    assert_eq!(g_nib(Nibble::Last, addr), 0xD);
    assert_eq!(g_nib(Nibble::Byte, addr), 0xCD);
    assert_eq!(g_nib(Nibble::Address, addr), 0xBCD);

    let addr: u16 = 0x6B1A;

    let second = g_nib(Nibble::Second, addr);
    let byte = g_nib(Nibble::Byte, addr);
    let thing = (second, byte);

    assert_eq!(thing, (0xb, 0x1a));
}

// fn get_nibbles(s: u8, e: u8, addr: u16) -> u16 {
//     //(1,2,add) => addr & 0FF0 >> 4

//     //addr >> e << s

//     // s :1, e: 2
//     //0xABCD

//     ((0xFFFF >> s) & addr) >> (4 * (3 - e))
// }

// //pixel on '█'
// //pixel off '░'
// pub fn draw(&self) {
//     let mut screen: String = String::new();

//     for i in 0..DISPLAY_HEIGHT {
//         for j in 0..DISPLAY_WIDTH {
//             if self.display[(i as u32 * DISPLAY_WIDTH as u32 + j as u32) as usize] != 0 {
//                 screen += "█";
//             } else {
//                 screen += "░";
//             }
//         }
//         screen += "\n";
//     }

//     println!("{}", screen);
// }
