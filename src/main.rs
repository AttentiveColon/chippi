use std::{thread, time};

mod lib;

fn main() {
    println!("Hello, world!");

    //TODO(lucypero): consider Chip8::from_rom() instead of new() and load_rom()
    let mut chip = lib::Chip8::new(lib::Computer::Normal);

    chip.load_rom("roms/test_opcode.ch8".into());

    print!("{:#06X?} ", 0x0);
    for (i, val) in chip.ram.into_iter().enumerate() {
        print!("{:02X?} ", val);
        if ((i + 1) % 32) == 0 {
            let indc = i + 1;
            print!("\n{:#06X?} ", indc);
        }
    }
    print!("\n");

    //run instruction on 60hz
    loop {
        chip.tick();
        chip.draw();
        //println!("{:?}", chip.display);

        // for (i, val) in chip.display.into_iter().enumerate() {
        //     print!("{} ", val);
        //     if ((i + 1) % 64) == 0 {
        //         print!("\n");
        //     }
        // }
        // print!("\n");
        thread::sleep(time::Duration::from_millis(100));
    }
}
