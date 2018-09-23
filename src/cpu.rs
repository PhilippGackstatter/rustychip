use self::Opcode::*;
use rand;
use std::fmt;

// Memory Map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

// 15 1-byte general purpose registers
// The 16th register is used for the ‘carry flag’

struct Register {
    v: Vec<u8>,
}

impl Register {
    fn new() -> Register {
        Register { v: vec![0; 16] }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "V0: {}\nV1: {}\nV2: {}\nV3: {}\nV4: {}\nV5: {}\nV6: {}\nV7: {}\nV8: {}\nV9: {}\nV10: {}\nV11: {}\nV12: {}\nV13: {}\nV14: {}\nV15: {}\n",
            self.v[0], self.v[1], self.v[2],self.v[3], self.v[4], self.v[5],
            self.v[6], self.v[7], self.v[8],self.v[9], self.v[10], self.v[11],
            self.v[12], self.v[13], self.v[14],self.v[15]
        );
        Ok(())
    }
}

#[derive(Debug)]
enum Opcode {
    Ignore,
    ClearScreen,
    Return,
    Jump(u16),
    SkipIfEqualAddress(u16, u16),
    SkipIfNotEqualAddress(u16, u16),
    SkipIfEqualRegister(u16, u16),
    SetRegister(u16, u16),
    SetIndexRegister(u16),
    CallSubroutine(u16),
    Display(u16, u16, u16),
    Add(u16, u16),
    AddAddressToRegister(u16, u16),
    Assign(u16, u16),
    AssignOr(u16, u16),
    AssignAnd(u16, u16),
    AssignXor(u16, u16),
    Subtract(u16, u16),
    LeastSigStoreAndShift(u16, u16),
    SetSubtract(u16, u16),
    MostSigStoreAndShift(u16, u16),
    SkipIfUnequalRegisters(u16, u16),
    Flow(u16),
    Rand(u16, u16),
    SkipIfKeyPressed(u16),
    SkipIfNotKeyPressed(u16),
    GetDelayTimer(u16),
    AwaitKeyPress(u16),
    SetDelayTimer(u16),
    SetSoundTimer(u16),
    AddToIndexRegister(u16),
    SetIndexRegisterToSpriteLocation(u16),
    StoreBinaryCodedDecimal(u16),
    RegisterDump(u16),
    RegisterLoad(u16),
    UNKNOWN(u16, u16, u16, u16),
}

pub struct CPU {
    // Memory
    memory: Vec<u8>,
    // Can have values between 0x000 and 0xFFF
    index_register: u16,
    // Can have values between 0x000 and 0xFFF
    pub gfx: Vec<u8>,
    program_counter: usize,
    register: Register,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    stack_pointer: usize,
    pub keypad: Vec<u8>,
    draw_flag: bool,
    debug_current_opcode: Opcode,
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}\n{:?}\n{}sp: {}\nI: {}\ndraw: {}\n",
            self.debug_current_opcode,
            self.keypad,
            self.register,
            self.stack_pointer,
            self.index_register,
            self.draw_flag
        );
        Ok(())
    }
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            memory: vec![0; 4096], // 0xfff + 1 = 0x1000
            keypad: vec![0; 16],
            stack: vec![0; 16],
            gfx: vec![0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            index_register: 0,
            program_counter: 0x200, // Start execution from this address
            draw_flag: false,
            register: Register::new(),
            debug_current_opcode: Ignore,
        };
        // Load the fontset into the first 512 bytes
        for i in 0..FONTSET.len() {
            cpu.memory[i] = FONTSET[i];
        }
        return cpu;
    }
    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        for i in 0..rom.len() {
            self.memory[0x200 + i] = rom[i];
        }
    }
    pub fn emulate_cycle(&mut self) -> bool {
        let opc = self.fetch();
        let decoded_opc = self.decode(opc);
        self.emulate(decoded_opc);
        return self.draw_flag;
    }
    fn fetch(&self) -> u16 {
        // Fetch 2 bytes to get the 16 bit opcode
        // Convert the u8s to u16s, so we can safely shift them by 8 bits
        let opcode1 = self.memory[self.program_counter] as u16;
        let opcode2 = self.memory[self.program_counter + 1] as u16;
        opcode1 << 8 | opcode2
    }
    fn decode(&self, opcode: u16) -> Opcode {
        let nib1 = (opcode & 0xF000) >> 12;
        let nib2 = (opcode & 0x0F00) >> 8;
        let nib3 = (opcode & 0x00F0) >> 4;
        let nib4 = opcode & 0x000F;

        let nnn = opcode & 0x0FFF;
        let nn = opcode & 0x00FF;

        // Map the u16 to the actual Opcode
        // https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description
        match (nib1, nib2, nib3, nib4) {
            (0x0, 0x0, 0x0, 0x0) => Ignore, // Can apparently be ignored
            (0x0, 0x0, 0xE, 0xE) => ClearScreen,
            (0x0, 0x0, 0xE, 0x0) => Return,
            (0x1, _, _, _) => Jump(nnn),
            (0x2, _, _, _) => CallSubroutine(nnn),
            (0x3, n1, _, _) => SkipIfEqualAddress(n1, nn),
            (0x4, n1, _, _) => SkipIfNotEqualAddress(n1, nn),
            (0x5, n1, n2, 0x0) => SkipIfEqualRegister(n1, n2),
            (0x6, n1, _, _) => SetRegister(n1, nn),
            (0x7, n1, _, _) => AddAddressToRegister(n1, nn),
            (0x8, n1, n2, 0x0) => Assign(n1, n2),
            (0x8, n1, n2, 0x1) => AssignOr(n1, n2),
            (0x8, n1, n2, 0x2) => AssignAnd(n1, n2),
            (0x8, n1, n2, 0x3) => AssignXor(n1, n2),
            (0x8, n1, n2, 0x4) => Add(n1, n2),
            (0x8, n1, n2, 0x5) => Subtract(n1, n2),
            (0x8, n1, n2, 0x6) => LeastSigStoreAndShift(n1, n2),
            (0x8, n1, n2, 0x7) => SetSubtract(n1, n2),
            (0x8, n1, n2, 0xE) => MostSigStoreAndShift(n1, n2),
            (0x9, n1, n2, 0x0) => SkipIfUnequalRegisters(n1, n2),
            (0xA, _, _, _) => SetIndexRegister(nnn),
            (0xB, _, _, _) => Flow(nnn),
            (0xC, n1, _, _) => Rand(n1, nn),
            (0xD, n1, n2, n3) => Display(n1, n2, n3),
            (0xE, n1, 0x9, 0xE) => SkipIfKeyPressed(n1),
            (0xE, n1, 0xA, 0x1) => SkipIfNotKeyPressed(n1),
            (0xF, n1, 0x0, 0x7) => GetDelayTimer(n1),
            (0xF, n1, 0x0, 0xA) => AwaitKeyPress(n1),
            (0xF, n1, 0x1, 0x5) => SetDelayTimer(n1),
            (0xF, n1, 0x1, 0x8) => SetSoundTimer(n1),
            (0xF, n1, 0x1, 0xE) => AddToIndexRegister(n1),
            (0xF, n1, 0x2, 0x9) => SetIndexRegisterToSpriteLocation(n1),
            (0xF, n1, 0x3, 0x3) => StoreBinaryCodedDecimal(n1),
            (0xF, n1, 0x5, 0x5) => RegisterDump(n1),
            (0xF, n1, 0x6, 0x5) => RegisterLoad(n1),
            _ => UNKNOWN(nib1, nib2, nib3, nib4),
        }
    }
    fn emulate(&mut self, opcode: Opcode) {
        // Decrement timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // println!("BEEP!");
            }
            self.sound_timer -= 1;
        }

        // Reset draw_flag
        self.draw_flag = false;

        match opcode {
            Ignore => (),
            ClearScreen => {
                // Clear Screen
                self.program_counter += 2;
            }
            Return => {
                self.program_counter = self.stack[self.stack_pointer] as usize;
                if self.stack_pointer > 0 {
                    println!("stackpointer is already 1");
                    self.stack_pointer -= 1;
                }
            }
            Jump(nnn) => {
                self.program_counter = nnn as usize;
            }
            CallSubroutine(nnn) => {
                self.stack[self.stack_pointer] = self.program_counter as u16;
                self.stack_pointer += 1;
                self.program_counter = nnn as usize;
            }
            SkipIfEqualAddress(x, nn) => {
                self.program_counter += if self.register.v[x as usize] == nn as u8 {
                    4
                } else {
                    2
                };
            }
            SkipIfNotEqualAddress(x, nn) => {
                self.program_counter += if self.register.v[x as usize] != nn as u8 {
                    4
                } else {
                    2
                };
            }
            SkipIfEqualRegister(x, y) => {
                let registers_are_equal =
                    self.register.v[x as usize] == self.register.v[y as usize];
                self.program_counter += if registers_are_equal { 4 } else { 2 };
            }
            SetRegister(x, nn) => {
                self.register.v[x as usize] = nn as u8;
                self.program_counter += 2;
            }
            AddAddressToRegister(x, nn) => {
                self.register.v[x as usize] = self.register.v[x as usize].wrapping_add(nn as u8);
                self.program_counter += 2;
            }
            Assign(x, y) => {
                self.register.v[x as usize] = self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignOr(x, y) => {
                self.register.v[x as usize] =
                    self.register.v[x as usize] | self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignAnd(x, y) => {
                self.register.v[x as usize] =
                    self.register.v[x as usize] & self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignXor(x, y) => {
                self.register.v[x as usize] =
                    self.register.v[x as usize] ^ self.register.v[y as usize];
                self.program_counter += 2;
            }
            Add(x, y) => {
                // Opcode 0x8XY4
                // Add VY to VX, set carry flag if overflow
                // Set carry flag if the result will be larger than 255
                if self.register.v[x as usize] > 0xff - self.register.v[y as usize] {
                    self.register.v[0xf] = 1;
                } else {
                    self.register.v[0xf] = 0;
                }

                let result = self.register.v[x as usize].wrapping_add(self.register.v[y as usize]);
                self.register.v[x as usize] = result;
                self.program_counter += 2;
            }
            Subtract(x, y) => {
                // TODO Unsure about this
                // VF is set to 0 when there's a borrow, and 1 when there isn't.
                // When VY is smaller/equal than VX, we can "safely" subtract, without underflowing
                // If it's greater then we underflow
                if self.register.v[y as usize] > self.register.v[x as usize] {
                    self.register.v[0xf] = 0;
                } else {
                    self.register.v[0xf] = 1;
                }

                let result = self.register.v[x as usize].wrapping_sub(self.register.v[y as usize]);
                self.register.v[x as usize] = result;
                self.program_counter += 2;
            }
            LeastSigStoreAndShift(x, _) => {
                // Stores the least significant bit of VX in VF and then shifts VX to the right by 1
                // Mask out everything but the least significant bit
                self.register.v[0xF] = self.register.v[x as usize] & 0x1;
                self.register.v[x as usize] >>= 1;
                self.program_counter += 2;
            }
            SetSubtract(x, y) => {
                // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                // If it's greater then we underflow
                if self.register.v[y as usize] < self.register.v[x as usize] {
                    // Underflow occurs
                    self.register.v[0xf] = 0;
                } else {
                    self.register.v[0xf] = 1;
                }

                let result = self.register.v[y as usize].wrapping_sub(self.register.v[x as usize]);
                self.register.v[x as usize] = result;

                self.program_counter += 2;
            }
            MostSigStoreAndShift(x, _) => {
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1
                let most_significant_bit = (self.register.v[x as usize] & 0x80) >> 7;
                self.register.v[0xf] = most_significant_bit;
                self.register.v[x as usize] <<= 1;
                self.program_counter += 2;
            }
            SkipIfUnequalRegisters(x, y) => {
                // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
                self.program_counter +=
                    if self.register.v[x as usize] == self.register.v[y as usize] {
                        4
                    } else {
                        2
                    }
                // if self.register.v[x as usize] == self.register.v[y as usize] {

                // }
            }
            SetIndexRegister(nnn) => {
                // Set I to nnn
                self.index_register = nnn;
                self.program_counter += 2;
            }
            Flow(nnn) => {
                // Jumps to the address NNN plus V0.
                self.index_register = (self.memory[nnn as usize] + self.register.v[0x0]) as u16;
                self.program_counter += 2;
            }
            Rand(x, nn) => {
                // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
                let random: u8 = rand::random();
                self.register.v[x as usize] = random & self.memory[nn as usize];
                self.program_counter += 2;
            }
            Display(x, y, n) => {
                // Coordinates at which the sprite is drawn
                let vx = self.register.v[x as usize] as u16;
                let vy = self.register.v[y as usize] as u16;

                // For every row
                for yline in 0..n {
                    // Read 8 pixels (represented through 1 byte) from memory starting at I
                    let byte = self.memory[(self.index_register + yline) as usize];
                    // display at x, y
                    // Before writing the byte, read the current value and check if pixels are flipped
                    // If so, set VF to 1, otherwise to 0

                    // Reset VF to 0
                    self.register.v[0xf] = 0;

                    // xline indicates the position in the line
                    for xline in 0..8 {
                        // 0x80 >> xline generates 128, 64, 32, .. which look like..
                        // 0x40 in binary is       0100 0000
                        // E.g. input 0x45 is      0100 0101
                        // through masking we get  0100 0000
                        let bit = byte & (0x80 >> xline);
                        // and we know bit 7 is set if the value is not decimal(0)
                        // because then the mask would've eliminated all 1s
                        let index = (vx + xline + (vy + yline) * 64) as usize;
                        if bit != 0 {
                            // Only if the pixel is turned off, set VF = 1
                            if self.gfx[index] == 1 {
                                self.register.v[0xf] = 1;
                            }
                            // Bit was set, so xor the current value
                            self.gfx[index] ^= 1;
                        }
                        // print!("{} ", self.gfx[index]);
                    }
                    // println!();
                }
                self.draw_flag = true;
                self.program_counter += 2;
            }
            SkipIfKeyPressed(x) => {
                //  Skip next instruction if key with the _value_ of Vx is pressed.
                if self.keypad[self.register.v[x as usize] as usize] != 0 {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            SkipIfNotKeyPressed(x) => {
                //  Skip next instruction if key with the _value_ of Vx is not pressed.
                 if self.keypad[self.register.v[x as usize] as usize] == 0 {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            GetDelayTimer(x) => {
                // Sets VX to the value of the delay timer.
                self.register.v[x as usize] = self.delay_timer;
                self.program_counter += 2;
            }
            AwaitKeyPress(x) => {
                let mut pressed_key = 20;

                for key in &self.keypad {
                    if *key == 1 {
                        pressed_key = *key;
                    }
                }

                if pressed_key != 20 {
                    self.register.v[x as usize] = pressed_key;
                    self.program_counter += 2;
                }
            }
            SetDelayTimer(x) => {
                // Sets VX to the value of the delay timer.
                self.delay_timer = self.register.v[x as usize];
                self.program_counter += 2;
            }
            SetSoundTimer(x) => {
                self.sound_timer = self.register.v[x as usize];
                self.program_counter += 2;
            }
            AddToIndexRegister(x) => {
                self.index_register += self.register.v[x as usize] as u16;
                self.program_counter += 2;
            }
            SetIndexRegisterToSpriteLocation(x) => {
                // Sets I to the location of the sprite for the character in VX.
                // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                self.index_register = (self.register.v[x as usize] * 5) as u16;
                self.program_counter += 2;
            }
            StoreBinaryCodedDecimal(pc) => {
                let x = (pc & 0x0F00) >> 8;
                let vx = self.register.v[x as usize];
                self.memory[self.index_register as usize] = vx / 100;
                self.memory[(self.index_register + 1) as usize] = (vx % 100) / 10;
                self.memory[(self.index_register + 1) as usize] = vx % 10;
                self.program_counter += 2;
            }
            RegisterDump(pc) => {
                let x = (pc & 0x0F00) >> 8;
                // Read V0 to VX (including VX) and write to memory starting at I
                for i in 0..x {
                    self.memory[(self.index_register + i) as usize] = self.register.v[i as usize];
                }
                // More complicated solution :D
                // for (i, v_reg) in self.register.v.iter().take((x + 1) as usize).enumerate() {
                //     self.memory[self.index_register as usize + i] = *v_reg;
                // }
                self.program_counter += 2;
            }
            RegisterLoad(pc) => {
                let x = (pc & 0x0F00) >> 8;
                // Read memory starting at I and copy to V0 to VX (including VX)
                for i in 0..x {
                    self.register.v[i as usize] = self.memory[(self.index_register + i) as usize];
                }
                self.program_counter += 2;
            }
            UNKNOWN(n1, n2, n3, n4) => println!("Unkown Instruction {} {} {} {}", n1, n2, n3, n4),
        }
        self.debug_current_opcode = opcode;
    }
}

// Every group of 5 bytes represent the corresponding character from 0 to F
// if drawn in binary, row by row
static FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];
