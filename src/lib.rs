#![allow(dead_code, non_snake_case)]

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

fn get_nibble(n: u8, addr: u16) -> u16 {
    match n {
        0 => (addr & 0xF000) >> 12,
        1 => (addr & 0x0F00) >> 8,
        2 => (addr & 0x00F0) >> 4,
        _ => addr & 0x000F,
    }
}

fn get_nibbles(s: u8, e: u8, addr: u16) -> u16 {
    //(1,2,add) => addr & 0FF0 >> 4

    //addr >> e << s

    // s :1, e: 2
    //0xABCD

    ((0xFFFF >> s) & addr) >> (4 * (3 - e))
}

pub struct Chip8 {
    ram: [u8; 4096],
    regs: [u8; 16],
    ireg: u16,
    dreg: u8,
    sreg: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    kb: [u8; 16],
    display: [u8; 64 * 32],
}

impl Chip8 {
    pub fn new(comp: Computer) -> Chip8 {
        Chip8 {
            ram: [0x0; 4096],
            regs: [0x0; 16],
            ireg: 0x00,
            dreg: 0x0,
            sreg: 0x0,
            sp: 0x0,
            stack: [0x00; 16],
            kb: [0x0; 16],
            display: [0x0; 64 * 32],
            pc: match comp {
                Computer::Normal => PROGRAM_START_LOCATION,
                Computer::Eti => ETI_PROGRAM_START_LOCATION,
            } as u16,
        }
    }

    pub fn load_rom(&mut self, path: String) {
        let the_file: Vec<u8> = std::fs::read(path).unwrap();

        println!("{:02X?}", the_file);

        //self.ram[]
        for (dst, src) in self
            .ram
            .iter_mut()
            .skip(PROGRAM_START_LOCATION)
            .zip(&the_file)
        {
            *dst = *src;
        }
        println!("{:02X?}", &self.ram[0x200..0x222]);
    }

    //executes the instruction on pc and changes all the state
    //ram[pc] + ram[pc + 1]
    pub fn tick(&mut self) {
        let instruction =
            ((self.ram[self.pc as usize] as u16) << 8) | self.ram[self.pc as usize + 1] as u16;
        println!("pc = {:#06X?}", instruction);

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
                    _ => self.SYS(get_nibbles(1, 3, instruction)),
                }
            }
            0x1 => self.JP(get_nibbles(1, 3, instruction)),
            0x2 => self.CALL(get_nibbles(1, 3, instruction)),
            0x3 => todo!(),
            0x4 => todo!(),
            0x5 => todo!(),
            0x6 => todo!(),
            0x7 => todo!(),
            0x8 => todo!(),
            0x9 => todo!(),
            0xA => todo!(),
            0xB => todo!(),
            0xC => todo!(),
            0xD => todo!(),
            0xE => todo!(),
            0xF => todo!(),
            _ => panic!(),
        }

        self.pc += 2;
    }

    /// 0nnn - SYS addr
    /// Jump to a machine code routine at nnn.
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented.
    /// It is ignored by modern interpreters.
    fn SYS(&mut self, addr: u16) {}

    /// 0e00 - CLS
    /// Clear the display.
    fn CLS(&mut self) {}

    /// 00ee - RET
    /// Return from a subroutine
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn RET(&mut self) {}

    /// 1nnn - JP addr
    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    pub fn JP(&mut self, addr: u16) {}

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack.
    /// The PC is then set to nnn.
    fn CALL(&mut self, addr: u16) {}

    /// 3xkk - SE Vx, Byte
    /// Skip the next instruction if Vx = kk.
    /// The interpreter compare register Vx to kk, and if they are equal, increments the program counter by 2.
    fn SE(&mut self, x: u8, kk: u8) {}

    /// 4xkk - SNE Vx, Byte
    /// Skip next instruction if Vx != kk.
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn SNE(&mut self, x: u8, kk: u8) {}

    /// 5xy0 - SER Vx, Vy
    /// Skip next instruction if Vx = Vy.
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn SER(&mut self, x: u8, y: u8) {}

    /// 6xkk - LD Vx, Byte
    /// Set Vx = kk
    /// The interpreter puts the value kk into register Vx.
    fn LD(&mut self, x: u8, kk: u8) {}

    /// 7xkk = ADD Vx, Byte
    /// Set Vx = Vx + kk
    /// Adds the value kk to the value of register Vx, then stores the result in Vx
    fn ADD(&mut self, x: u8, kk: u8) {}

    /// 8xy0 - LDR Vx, Vy
    /// Set Vx = Vy
    /// Stores the value of register Vy in register Vx.
    fn LDR(&mut self, x: u8, y: u8) {}

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy
    /// Performs a bitwise Or on the values of Vx and Vy, then stores the result in Vx.
    fn OR(&mut self, x: u8, y: u8) {}

    /// 8xy2 - AND Vx, Vy
    /// Set Vx = Vx AND Vy
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    fn AND(&mut self, x: u8, y: u8) {}

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    fn XOR(&mut self, x: u8, y: u8) {}

    /// 8xy4 = ADDR Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits VF is set to 1, Otherwise 0.
    /// Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn ADDR(&mut self, x: u8, y: u8) {}

    /// 8xy5 = SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted form Vx, and the results stored in Vx.
    fn SUB(&mut self, x: u8, y: u8) {}

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx SHR 1
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn SHR(&mut self, x: u8, y: u8) {}

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the result is stored in Vx.
    fn SUBN(&mut self, x: u8, y: u8) {}

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx SHL 1
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is multiplied by 2.
    fn SHL(&mut self, x: u8, y: u8) {}

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction is Vx != Vy
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn SNER(&mut self, x: u8, y: u8) {}

    /// Annn - LDI I, addr
    /// Set I = nnn.
    /// The value of register I is set to nnn.
    fn LDI(&mut self, addr: u16) {}

    /// Bnnn JPO V0, addr
    /// Jump to location nnn + V0
    /// The program counter is set to nnn plus the value of V0
    fn JPO(&mut self, addr: u16) {}

    /// Cxkk - RND Vx, Byte
    /// Set Vx = random byte AND kk
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    /// The results are stored in Vx.
    fn RND(&mut self, x: u8, kk: u8) {}

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed
    /// as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels
    /// to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the
    /// coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more
    /// information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    fn DRW(&mut self, x: u8, y: u8, n: u8) {}

    /// Ex9E - SKPK Vx
    /// Skip next instruction if key with the value of Vx is pressed
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC in increased by 2.
    fn SKPK(&mut self, x: u8) {}

    /// ExA1 - SKNPK Vx
    /// Skip the next instruction if key with the value of Vx is not pressed.
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    fn SKNPK(&mut self, x: u8) {}

    /// Fx07 - LDT Vx, DT
    /// Set Vx = delay timer value
    /// The value of DT is placed into Vx.
    fn LDT(&mut self, x: u8) {}

    /// Fx0A - LDK Vx, K
    /// Wait for a key press, store the value of the key in Vx
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn LDK(&mut self, x: u8) {}

    /// Fx15 - LDD DT, Vx
    /// Set delay timer = Vx
    /// DT is set equal to value of Vx.
    fn LDD(&mut self, x: u8) {}

    /// Fx18 - LDS ST, Vx
    /// Set sound timer = Vx
    /// ST is set equal to the value of Vx.
    fn LDS(&mut self, x: u8) {}

    /// Fx1E - ADDI I, Vx
    /// Set I = I + Vx
    /// The values of I and Vx are added, and the results are stored in I.
    fn ADDI(&mut self, x: u8) {}

    /// Fx29 - LDF F, Vx
    /// Set I = Location of sprite for digit Vx.
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    fn LDF(&mut self, x: u8) {}

    /// Fx33 - LDB B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2
    /// The integer takes the decimal value of Vx, and places the hundreds digit in memory at location I, the tens digit in location I+1,
    /// and the ones digits at location I+2.
    fn LDB(&mut self, x: u8) {}

    /// Fx55 - LDIX [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    /// The interpreter copies the values of registers v0 through Vx ihnto memory, starting at the address in I.
    fn LDIX(&mut self, x: u8) {}

    /// Fx65 - LDRX Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn LDRX(&mut self, x: u8) {}
}

#[test]
fn tests() {
    let addr: u16 = 0xABCD;

    assert_eq!(get_nibble(0, addr), 0xA);
    assert_eq!(get_nibble(1, addr), 0xB);
    assert_eq!(get_nibble(2, addr), 0xC);
    assert_eq!(get_nibble(3, addr), 0xD);
}

#[test]
fn test_get_nibbles() {
    let addr: u16 = 0xABCD;
    assert_eq!(get_nibbles(0, 1, addr), 0xAB);
    assert_eq!(get_nibbles(1, 2, addr), 0xBC);
    assert_eq!(get_nibbles(1, 1, addr), 0xB);
    assert_eq!(get_nibbles(1, 3, addr), 0xBCD);
}

#[test]
fn test_g_nibs() {
    let addr: u16 = 0xABCD;
    assert_eq!(g_nib(Nibble::Second, addr), 0xB);
    assert_eq!(g_nib(Nibble::Third, addr), 0xC);
    assert_eq!(g_nib(Nibble::Last, addr), 0xD);
    assert_eq!(g_nib(Nibble::Byte, addr), 0xCD);
    assert_eq!(g_nib(Nibble::Address, addr), 0xBCD);
}
