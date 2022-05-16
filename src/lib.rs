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

        let instruction = ((self.ram[self.pc as usize] as u16) << 8) | self.ram[self.pc as usize + 1] as u16;
        println!("pc = {:#06X?}", instruction);

        //get the first nibble and pattern match it
        match get_nibble(0, instruction) {
            0x0 => { //this can be 1 of 3 instr
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
            _ => panic!()
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
