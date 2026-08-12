mod chip8_const;
mod chip8_font;
mod chip8_instr;

use chip8_const::*;
use chip8_instr::Chip8Instr;

use chip8_font::FONTSET;
use chip8_font::FONTSET_SIZE;
use chip8_font::FONTSET_START_ADDR;

pub struct Chip8 {
    // Each pixel can either be on/true (white) or off/false (black)
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Registers
    pc: u16,
    v_reg: [u8; NUM_REGS], // 16 V registers
    i_reg: u16,            // index register

    // Stack holds return addresses, sp is an index into the stack
    sp: u8, // Holds the index of the top of the stack (last unused position)
    stack: [u16; STACK_SIZE],

    ram: [u8; RAM_SIZE],

    // Timers
    dt: u8,
    st: u8,

    // Input keys
    keys: [bool; NUM_KEYS],
    keys_prev: [bool; NUM_KEYS],  // maintain 1 frame of history
    state_fx0a_started: bool,

    // Only allow 1 draw sprite operation per screen refresh (60Hz)
    wait_for_v_blank: bool,

    // Quirks
    quirk_8xy6: bool,
    quirk_8xye: bool,
    quirk_fx55: bool,
    quirk_fx65: bool,
}

impl Chip8 {
    // Constructor
    pub fn new(quirk_8xy6: bool, quirk_8xye: bool, quirk_fx55: bool, quirk_fx65: bool) -> Self {
        let mut c8 = Self {
            pc: PC_START_ADDR,
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            dt: 0,
            st: 0,
            keys: [false; NUM_KEYS],
            keys_prev: [false; NUM_KEYS],
            state_fx0a_started: false,
            wait_for_v_blank: false,
            quirk_8xy6,
            quirk_8xye,
            quirk_fx55,
            quirk_fx65,
        };

        // Copy fonts into RAM
        c8.ram[FONTSET_START_ADDR..FONTSET_SIZE].copy_from_slice(&FONTSET);

        c8
    }

    // Stack Methods
    fn stack_push(&mut self, addr: u16) {
        self.stack[self.sp as usize] = addr;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // Instructions

    // CLS
    fn op_00e0(&mut self, _instr: Chip8Instr) {
        self.screen.fill(false);
    }

    // RET
    fn op_00ee(&mut self, _instr: Chip8Instr) {
        self.pc = self.stack_pop();
    }

    // JMP
    fn op_1nnn(&mut self, instr: Chip8Instr) {
        self.pc = instr.nnn();
    }

    // JAL
    fn op_2nnn(&mut self, instr: Chip8Instr) {
        // Save current PC to the stack before going there
        self.stack_push(self.pc);
        self.op_1nnn(instr);
    }

    // SEQI
    fn op_3xnn(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX == NN
        if self.v_reg[instr.reg_x()] == instr.nn() {
            self.pc += 2;
        }
    }

    // SNEI
    fn op_4xnn(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX != NN
        if self.v_reg[instr.reg_x()] != instr.nn() {
            self.pc += 2;
        }
    }

    // SEQ
    fn op_5xy0(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX == VY
        if self.v_reg[instr.reg_x()] == self.v_reg[instr.reg_y()] {
            self.pc += 2;
        }
    }

    // LI
    fn op_6xnn(&mut self, instr: Chip8Instr) {
        // Load NN into VX
        self.v_reg[instr.reg_x()] = instr.nn();
    }

    // ADDI
    fn op_7xnn(&mut self, instr: Chip8Instr) {
        // Add NN to VX
        // Wrapping add to avoid panic on overflow
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_x()].wrapping_add(instr.nn());
    }

    // MV
    fn op_8xy0(&mut self, instr: Chip8Instr) {
        // Copy register VY into VX
        self.v_reg[instr.reg_x()] = self.v_reg[instr.reg_y()];
    }

    // SEOR
    fn op_8xy1(&mut self, instr: Chip8Instr) {
        // Set VX to VX | VY
        self.v_reg[instr.reg_x()] |= self.v_reg[instr.reg_y()];

        // Quirk: Reset VF to 0
        self.v_reg[0xF] = 0;
    }

    // SEAND
    fn op_8xy2(&mut self, instr: Chip8Instr) {
        // Set VX to VX & VY
        self.v_reg[instr.reg_x()] &= self.v_reg[instr.reg_y()];

        // Quirk: Reset VF to 0
        self.v_reg[0xF] = 0;
    }

    // SEXOR
    fn op_8xy3(&mut self, instr: Chip8Instr) {
        // Set VX to VX ^ VY
        self.v_reg[instr.reg_x()] ^= self.v_reg[instr.reg_y()];

        // Quirk: Reset VF to 0
        self.v_reg[0xF] = 0;
    }

    // ADD
    fn op_8xy4(&mut self, instr: Chip8Instr) {
        // VX = VX + VY
        // VF set to 1 on overflow
        let (result, overflow) =
            self.v_reg[instr.reg_x()].overflowing_add(self.v_reg[instr.reg_y()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if overflow { 1 } else { 0 };
    }

    // SUB
    fn op_8xy5(&mut self, instr: Chip8Instr) {
        // VX = VX - VY
        let (result, borrow) = self.v_reg[instr.reg_x()].overflowing_sub(self.v_reg[instr.reg_y()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if borrow { 0 } else { 1 };
    }

    // SRL
    fn op_8xy6(&mut self, instr: Chip8Instr) {
        // VX = VY >> 1
        // VF = LSB of VY

        // Quirk on -> always operate on VX
        let reg_x = instr.reg_x();
        let reg_y = if self.quirk_8xy6 {reg_x} else {instr.reg_y()};

        // In case reg_x and reg_y are the same !!
        let orig_val = self.v_reg[reg_y];

        self.v_reg[reg_x] = orig_val >> 1;
        self.v_reg[0xF] = orig_val & 1;
    }

    // SUB2
    fn op_8xy7(&mut self, instr: Chip8Instr) {
        // VX = VY - VX
        let (result, borrow) = self.v_reg[instr.reg_y()].overflowing_sub(self.v_reg[instr.reg_x()]);

        self.v_reg[instr.reg_x()] = result;
        self.v_reg[0xF] = if borrow { 0 } else { 1 };
    }

    // SLL
    fn op_8xye(&mut self, instr: Chip8Instr) {
        // VX = VY << 1
        // VF = LSB of VY

        // Quirk on -> always operate on VX
        let reg_x = instr.reg_x();
        let reg_y = if self.quirk_8xye {reg_x} else {instr.reg_y()};

        // In case reg_x and reg_y are the same !!
        let orig_val = self.v_reg[reg_y];

        self.v_reg[reg_x] = orig_val << 1;
        self.v_reg[0xF] = (orig_val & 0x80) >> 7;
    }

    // SNE
    fn op_9xy0(&mut self, instr: Chip8Instr) {
        // Skip following instruction if VX != VY
        if self.v_reg[instr.reg_x()] != self.v_reg[instr.reg_y()] {
            self.pc += 2;
        }
    }

    // SMI
    fn op_annn(&mut self, instr: Chip8Instr) {
        // Store NNN into i_reg
        self.i_reg = instr.nnn();
    }

    // LJMP
    fn op_bnnn(&mut self, instr: Chip8Instr) {
        // PC = NNN + V0
        self.pc = instr.nnn() + self.v_reg[0] as u16;
    }

    // SRND
    fn op_cxnn(&mut self, instr: Chip8Instr) {
        // reg_x = random number & 0xNN
        let random: u8 = rand::random();
        self.v_reg[instr.reg_x()] = random & instr.nn();
    }

    // DSPR
    fn op_dxyn(&mut self, instr: Chip8Instr) {
        // Draw a sprite with N bytes (height) to the screen starting at (VX, VY)
        let num_bytes = instr.n() as usize;
        let sprite_start = self.i_reg as usize;
        let mut collision = false;

        // Loop over all bytes
        let mut y = (self.v_reg[instr.reg_y()] % SCREEN_HEIGHT as u8) as usize;
        for byte in &self.ram[sprite_start..sprite_start + num_bytes] {
            // Loop over all bits in the byte (each sprite is 8 pixels wide)
            let mut x = (self.v_reg[instr.reg_x()] % SCREEN_WIDTH as u8) as usize;
            for bit_idx in 0..8 {
                // If the pixel in the sprite is set and in range
                if byte & (0x80 >> bit_idx) != 0 && x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
                    // Set collision bit
                    collision |= self.screen[x + SCREEN_WIDTH * y];

                    // Draw to screen
                    self.screen[x + SCREEN_WIDTH * y] ^= true;
                }

                x = x + 1;
            }

            y = y + 1;
        }

        self.v_reg[0xF] = if collision { 1 } else { 0 };
        self.wait_for_v_blank = true;
    }

    // SKP
    fn op_ex9e(&mut self, instr: Chip8Instr) {
        // Skip next instr if the key in VX is pressed
        let key_code = self.v_reg[instr.reg_x()] as usize;
        if self.keys.get(key_code).is_some_and(|x| *x) {
            self.pc += 2;
        }
    }

    // SKNP
    fn op_exa1(&mut self, instr: Chip8Instr) {
        // Skip next instr if the key in VX is not pressed
        let key_code = self.v_reg[instr.reg_x()] as usize;
        if self.keys.get(key_code).is_some_and(|x| !*x) {
            self.pc += 2;
        }
    }

    // SDT
    fn op_fx07(&mut self, instr: Chip8Instr) {
        // Store current delay timer value in VX
        self.v_reg[instr.reg_x()] = self.dt;
    }

    // WKP
    fn op_fx0a(&mut self, instr: Chip8Instr) {
        // First time this instruction is run, set all keys to not pressed
        // Need to wait for a brand new key press
        if !self.state_fx0a_started {
            self.keys_prev.fill(false);
            self.keys.fill(false);
            self.state_fx0a_started = true;
        }

        for (i, (key_prev, key)) in self.keys_prev.iter().zip(&self.keys).enumerate() {
            if *key_prev && !*key {
                // Key was pressed in last frame, but not current
                self.v_reg[instr.reg_x()] = i as u8;
                self.state_fx0a_started = false;
                return;
            }
        }

        // Keep blocking until condition is met
        self.pc -= 2;
    }

    // STDT
    fn op_fx15(&mut self, instr: Chip8Instr) {
        // Set delay timer to VX
        self.dt = self.v_reg[instr.reg_x()];
    }

    // STST
    fn op_fx18(&mut self, instr: Chip8Instr) {
        // Set sound timer to VX
        self.st = self.v_reg[instr.reg_x()];
    }

    // ADDX
    fn op_fx1e(&mut self, instr: Chip8Instr) {
        // i_reg += VX
        (self.i_reg, _) = self.i_reg.overflowing_add(self.v_reg[instr.reg_x()] as u16);
    }

    fn op_fx29(&mut self, instr: Chip8Instr) {
        // Set the I register to font address of the hex character in VX
        let hex_char = (self.v_reg[instr.reg_x()] & 0x0F) as u16;
        self.i_reg = (FONTSET_START_ADDR as u16) + (hex_char * 5);
    }

    fn op_fx33(&mut self, instr: Chip8Instr) {
        // Store BCD representation of VX. Hundreds goes into I, tens into I+1, ones into I+2
        let mut value = self.v_reg[instr.reg_x()];

        for i in (0..=2).rev() {
            self.ram[(self.i_reg + i) as usize] = value % 10;
            value /= 10;
        }
    }

    fn op_fx55(&mut self, instr: Chip8Instr) {
        // Write registers V0-VX into memory from I-I+X
        let i_reg_start = self.i_reg;
        for i in 0..=instr.reg_x() {
            self.ram[self.i_reg as usize] = self.v_reg[i];
            self.i_reg += 1;
        }

        // Quirk on -> don't modify i_reg
        if self.quirk_fx55 {
            self.i_reg = i_reg_start
        }
    }

    fn op_fx65(&mut self, instr: Chip8Instr) {
        // Read registers V0-VX from memory at I-I+X
        let i_reg_start = self.i_reg;
        for i in 0..=instr.reg_x() {
            self.v_reg[i] = self.ram[self.i_reg as usize];
            self.i_reg += 1;
        }

        // Quirk on -> don't modify i_reg
        if self.quirk_fx65 {
            self.i_reg = i_reg_start
        }
    }

    // Instruction Handling
    fn fetch(&mut self) -> Chip8Instr {
        // Chip-8 is a big-endian machine
        // Retrieve the byte at pc and pc+1 into the u16
        let instr =
            ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[self.pc as usize + 1] as u16);
        self.pc += 2;
        Chip8Instr { bits: instr }
    }

    fn execute(&mut self, instr: Chip8Instr) {
        match (instr.first(), instr.second(), instr.third(), instr.fourth()) {
            (0x0, 0x0, 0x0, 0x0) => return,
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(instr),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(instr),
            (0x1, _, _, _) => self.op_1nnn(instr),
            (0x2, _, _, _) => self.op_2nnn(instr),
            (0x3, _, _, _) => self.op_3xnn(instr),
            (0x4, _, _, _) => self.op_4xnn(instr),
            (0x5, _, _, 0x0) => self.op_5xy0(instr),
            (0x6, _, _, _) => self.op_6xnn(instr),
            (0x7, _, _, _) => self.op_7xnn(instr),
            (0x8, _, _, 0x0) => self.op_8xy0(instr),
            (0x8, _, _, 0x1) => self.op_8xy1(instr),
            (0x8, _, _, 0x2) => self.op_8xy2(instr),
            (0x8, _, _, 0x3) => self.op_8xy3(instr),
            (0x8, _, _, 0x4) => self.op_8xy4(instr),
            (0x8, _, _, 0x5) => self.op_8xy5(instr),
            (0x8, _, _, 0x6) => self.op_8xy6(instr),
            (0x8, _, _, 0x7) => self.op_8xy7(instr),
            (0x8, _, _, 0xE) => self.op_8xye(instr),
            (0x9, _, _, 0) => self.op_9xy0(instr),
            (0xA, _, _, _) => self.op_annn(instr),
            (0xB, _, _, _) => self.op_bnnn(instr),
            (0xC, _, _, _) => self.op_cxnn(instr),
            (0xD, _, _, _) => self.op_dxyn(instr),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(instr),
            (0xE, _, 0xA, 0x1) => self.op_exa1(instr),
            (0xF, _, 0x0, 0x7) => self.op_fx07(instr),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(instr),
            (0xF, _, 0x1, 0x5) => self.op_fx15(instr),
            (0xF, _, 0x1, 0x8) => self.op_fx18(instr),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(instr),
            (0xF, _, 0x2, 0x9) => self.op_fx29(instr),
            (0xF, _, 0x3, 0x3) => self.op_fx33(instr),
            (0xF, _, 0x5, 0x5) => self.op_fx55(instr),
            (0xF, _, 0x6, 0x5) => self.op_fx65(instr),
            _ => unimplemented!("Unimplemented opcode: {:?}", instr),
        }
    }

    // Runs at Clock Rate
    pub fn tick(&mut self) {
        if self.wait_for_v_blank {
            return;
        }

        // Fetch
        let instr = self.fetch();

        // Decode and Execute
        self.execute(instr);
    }

    // Runs at 60Hz
    pub fn tick_timers(&mut self) -> bool {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);

        if self.st > 0 {
            return true;
        }

        return false;
    }

    pub fn notify_vblank(&mut self) {
        self.wait_for_v_blank = false;
    }

    // Load ROM
    pub fn load(&mut self, data: &[u8]) {
        let start = PC_START_ADDR as usize;
        let end = start + data.len() as usize;
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn set_key(&mut self, key: usize, state: bool) {
        // Copy previous history
        self.keys_prev.copy_from_slice(&self.keys);
        self.keys[key] = state;
    }
}
