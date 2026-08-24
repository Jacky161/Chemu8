# Chemu8

Chemu8 is a Chip8 emulator written in Rust. A desktop version is available, with a planned browser version coming soon.

## Chemu8 Desktop

Chemu8 Desktop can be compiled and ran on Windows/MacOS/Linux. The UI can be interacted with using the indicated number keys. If you prefer, you may use CLI arguments to pass the ROM file, as well as enable/disable quirks. The UI also features these same options.

### Compiling and Running

Download the latest release from the [releases page](https://github.com/Jacky161/Chemu8/releases) or clone the source code to compile it yourself.

To compile the code yourself, `cd` into the `chemu8_desktop` folder and run `cargo build` or `cargo build --release` for a debug/release build respectively. You will need to have Rust installed on your system. The compiled binary is located in `./target/{debug/release}/chemu8_desktop`.

Use the `--help` flag to see all the available CLI arguments.

## ROMs

Find ROMs to use with this emulator here!

https://github.com/kripod/chip8-roms

## Resources

Here are the links to the resources I used to develop this emulator.

- https://github.com/aquova/chip8-book
- https://austinmorlan.com/posts/chip8_emulator/
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- https://github.com/mattmikolay/chip-8/wiki
- https://github.com/Timendus/chip8-test-suite
