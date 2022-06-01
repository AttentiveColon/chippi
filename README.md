# Chippi: CHIP-8 Emulator
Emulator / interpreter implementation of the [CHIP-8 spec](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)

![splash screen](assets/splash.gif)

![bricks](assets/bricks.gif)

![blinky](assets/blinky.gif)

### Usage
(all arguments are optional)

`cargo run --release [rom_path] [speed_multiplier] [rainbow_mode]`

**Examples:**

Running blinky at speed 5 with rainbow mode:

`cargo run --release roms/blinky.ch8 5 true`

Running default splash screen:

`cargo run --release`

### Controls
```
8 => Decrease speed
9 => Increase speed
0 => Change Color

Esc => Exit

1|2|3|C|  =>  |1|2|3|4|
4|5|6|D|  =>  |Q|W|E|R|
7|8|9|E|  =>  |A|S|D|F|
A|0|B|F|  =>  |Z|X|C|V|
```
