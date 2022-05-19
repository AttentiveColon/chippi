use std::{thread, time};
use windows::Win32::System::Console::*;
use windows::Win32::Foundation::BOOL;

mod lib;

fn main() {
    unsafe {
        let hStdin = GetStdHandle(STD_INPUT_HANDLE).expect("Couldn't get StdHandle");
        let res = SetConsoleMode(hStdin, ENABLE_PROCESSED_OUTPUT);
        if !res.as_bool() {
            panic!("setconsolemode failed");
        }

        for i in 0..100 {
            let mut irInBuf: [INPUT_RECORD; 128] = [INPUT_RECORD::default(); 128];
            let mut numberOfRecords = 0;

            let res = ReadConsoleInputA(hStdin, &mut irInBuf, &mut numberOfRecords);
            assert_eq!(res.as_bool(), true);

            for r in 0..numberOfRecords as usize {
                let record = irInBuf[r];

                // pub struct KEY_EVENT_RECORD {
                //     pub bKeyDown: super::super::Foundation::BOOL,
                //     pub wRepeatCount: u16,
                //     pub wVirtualKeyCode: u16,
                //     pub wVirtualScanCode: u16,
                //     pub uChar: KEY_EVENT_RECORD_0,
                //     pub dwControlKeyState: u32,
                // }

                let ker = record.Event.KeyEvent;

                let key_c = char::from(ker.uChar.AsciiChar.0);

                println!("{:?}", ker.bKeyDown);

                // println!(
                //     "key record: key down: {:?}, repeat count:{}, {} {} {}",
                //     ker.bKeyDown,
                //     ker.wRepeatCount,
                //     ker.wVirtualKeyCode,
                //     ker.wVirtualScanCode,
                //     char::from(ker.uChar.AsciiChar.0)
                // );

                if key_c == 'q' {
                    panic!();
                }
            }
        }
    }

    //TODO(lucypero): consider Chip8::from_rom() instead of new() and load_rom()
    let mut chip = lib::Chip8::new(lib::Computer::Normal);

    chip.load_rom("roms/clock.ch8".into());

    //run instruction on 60hz
    loop {
        chip.tick();
        chip.draw();
        thread::sleep(time::Duration::from_millis(16));
    }
}
