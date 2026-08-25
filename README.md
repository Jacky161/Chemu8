# Chemu8

Chemu8 is a Chip8 emulator written in Rust. A desktop version is available, with a planned browser version coming soon.

## Chemu8 Desktop

Chemu8 Desktop can be compiled and ran on Windows/MacOS/Linux. The UI can be interacted with using the indicated number keys. If you prefer, you may use CLI arguments to pass the ROM file, as well as enable/disable quirks. The UI also features these same options.

### Compiling and Running

Download the latest release from the [releases page](https://github.com/Jacky161/Chemu8/releases) or clone the source code to compile it yourself. You will also need to install SDL2 if you are on MacOS or Linux.

On MacOS, run `brew install sdl2 sdl2_gfx sdl2_ttf`. Use your distribution's package manage to install the corresponding packages on Linux.

To compile the code yourself, `cd` into the `chemu8_desktop` folder and run `cargo build` or `cargo build --release` for a debug/release build respectively. You will need to have Rust installed on your system. The compiled binary is located in `./target/{debug/release}/chemu8_desktop`.

Use the `--help` flag to see all the available CLI arguments.

## Configuration

Currently, the only configurable option is quirks. By default, all quirks are enabled to maximize compatability out of the box.

Quirks stem from minor differences in the implementations of particular Chip8 instructions. The most common reference, [Cowgod](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) has some inaccuracies in the descriptions of `8xy6`, `8xye`, `fx55`, and `fx65`. Some games have been implemented assuming this inaccurate behaviour, leading to issues when playing them with emulators that have been implemented with corrected references.

Setting quirks enabled in Chemu8 (the default) will have the affected instructions behave according to the Cowgod reference. Disabling quirks will use the accurate behaviour as seen in corrected references (such as in [Matthew Mikolay's reference](https://github.com/mattmikolay/chip-8/wiki)).

Enabling or disabling quirks may be required for certain games to behave properly.

## Controls

The Chip8 features a keypad with 16 buttons, arranged in 4 rows of 4 buttons each. The mapping is standard across Chip8 emulators, and is as follows.

```
Keypad       Keyboard
+-+-+-+-+    +-+-+-+-+
|1|2|3|C|    |1|2|3|4|
+-+-+-+-+    +-+-+-+-+
|4|5|6|D|    |Q|W|E|R|
+-+-+-+-+ => +-+-+-+-+
|7|8|9|E|    |A|S|D|F|
+-+-+-+-+    +-+-+-+-+
|A|0|B|F|    |Z|X|C|V|
+-+-+-+-+    +-+-+-+-+
```

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
