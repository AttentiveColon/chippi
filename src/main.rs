
use std::{thread, time};


mod lib;


fn main() {
    println!("Hello, world!");

    //TODO(lucypero): consider Chip8::from_rom() instead of new() and load_rom()
    let mut chip = lib::Chip8::new(lib::Computer::Normal);

    
    chip.load_rom("roms/random_num_test.ch8".into());

    //run instruction on 60hz
    loop  {
        chip.tick();
        thread::sleep(time::Duration::from_millis(16));
    }
   
}