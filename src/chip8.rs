#![allow(dead_code, non_snake_case)]

use macroquad::rand;
use macroquad::file;

#[derive(Debug)]
enum Chip8Error {
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

fn get_bits(addr: u16) -> (u8, u8, u8, u8, u8, u8, u16) {
    (
        ((addr & 0xF000) >> 12) as u8,
        (addr & 0xF) as u8,
        ((addr >> 8) & 0xf) as u8,
        ((addr >> 4) & 0xf) as u8,
        (addr & 0xf) as u8,
        (addr & 0xff) as u8,
        addr & 0xFFF,
    )
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
}

impl Chip8 {
    pub async fn from_rom(comp: Computer, rom: String) -> Chip8 {
        let mut chip8 = Chip8 {
            ram: load_text(TEXT_MEMORY_START),
            regs: [0x0; 16],
            ireg: 0x00,
            dreg: 0x00,
            sreg: 0x0,
            pc: match comp {
                Computer::Normal => PROGRAM_START_LOCATION,
                Computer::Eti => ETI_PROGRAM_START_LOCATION,
            } as u16,
            sp: 0x0,
            stack: [0x00; 16],
            kb: [0x0; 16],
            display: [0; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
        };
        chip8.load_rom(rom.clone()).await.expect(&format!("Couldn't Load Rom. path: {rom}"));
        chip8
    }

    async fn load_rom(&mut self, path: String) -> Result<(), Chip8Error> {
        let the_file: Vec<u8> = file::load_file(&path).await.map_err(|_| Chip8Error::FileNotFound)?;

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

        let (first, last, x, y, n, kk, nnn) = get_bits(instruction);

        match first {
            0x0 => match instruction {
                0x00E0 => self.CLS(),
                0x00EE => self.RET(),
                _ => self.SYS(nnn),
            },
            0x1 => self.JP(nnn),
            0x2 => self.CALL(nnn),
            0x3 => self.SE(x, kk),
            0x4 => self.SNE(x, kk),
            0x5 => self.SER(x, y),
            0x6 => self.LD(x, kk),
            0x7 => self.ADD(x, kk),
            0x8 => match last {
                0x0 => self.LDR(x, y),
                0x1 => self.OR(x, y),
                0x2 => self.AND(x, y),
                0x3 => self.XOR(x, y),
                0x4 => self.ADDR(x, y),
                0x5 => self.SUB(x, y),
                0x6 => self.SHR(x, y),
                0x7 => self.SUBN(x, y),
                0xE => self.SHL(x, y),
                _ => panic!(),
            },
            0x9 => self.SNER(x, y),
            0xA => self.LDI(nnn),
            0xB => self.JPO(nnn),
            0xC => self.RND(x, kk),
            0xD => self.DRW(x, y, n),
            0xE => match kk {
                0x9E => self.SKPK(x),
                0xA1 => self.SKNPK(x),
                _ => panic!(),
            },
            0xF => match kk {
                0x07 => self.LDT(x),
                0x0A => self.LDK(x),
                0x15 => self.LDD(x),
                0x18 => self.LDS(x),
                0x1E => self.ADDI(x),
                0x29 => self.LDF(x),
                0x33 => self.LDB(x),
                0x55 => self.LDIX(x),
                0x65 => self.LDRX(x),
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

    /// 00e0 - CLS
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
    fn JP(&mut self, addr: u16) {
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
        self.ireg = addr & 0xFFF;
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
        self.regs[x as usize] = rand::gen_range(0u8, 255u8) & kk;
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
        let mut set_collision = false;
        let posX = self.regs[x as usize] as u32;
        let posY = self.regs[y as usize] as u32;
        for i in 0..n as u32 {
            let byte_index = self.ireg as usize + i as usize;
            let mut current_byte = self.ram[byte_index];

            for j in 0..8 as u32 {
                let position = ((posY + i) % DISPLAY_HEIGHT as u32) * DISPLAY_WIDTH as u32
                    + (posX + j) % DISPLAY_WIDTH as u32;

                let current_bit = (current_byte & 0x80) >> 7;

                if self.display[position as usize] != 0
                    && self.display[position as usize] ^ current_bit == 0
                {
                    set_collision = true;
                }
                self.display[position as usize] ^= current_bit;
                current_byte <<= 1;
            }
        }
        if set_collision {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
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
        for (i, val) in self.kb.iter().enumerate() {
            if val != &0 {
                self.regs[x as usize] = i as u8;
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