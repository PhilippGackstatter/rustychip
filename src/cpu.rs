use self::Opcode::*;
use rand;

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
        Register {
            v: Vec::with_capacity(16),
        }
    }
}

enum Opcode {
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
    program_counter: usize,
    // 64 x 32 pixel video array, black (0) or white (1)
    gfx: Vec<u8>,
    register: Register,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    stack_pointer: usize,
    keypad: Vec<u8>,
    drawFlag: bool,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            memory: Vec::with_capacity(4096), // 0xfff + 1 = 0x1000
            gfx: Vec::with_capacity(64 * 32),
            keypad: Vec::with_capacity(16),
            stack: Vec::with_capacity(16),
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            index_register: 0,
            program_counter: 0x200, // Start execution from this address
            drawFlag: false,
            register: Register::new(),
        };
        // Load the fontset into the first 512 bytes
        for i in 0..0x200 {
            cpu.memory[i] = FONTSET[i];
        }
        return cpu;
    }
    pub fn emulate_cycle(&mut self) {
        let opc = self.fetch();
        let decoded_opc = self.decode(opc);
        self.emulate(decoded_opc);
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
        match opcode {
            ClearScreen => {
                // Clear Screen
                self.program_counter += 2;
            }
            Return => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer] as usize;
            }
            Jump(nnn) => {
                self.program_counter = nnn as usize;
            }
            CallSubroutine(nnn) => {
                self.stack[self.stack_pointer] = self.program_counter as u16;
                self.stack_pointer += 1;
                self.program_counter = nnn as usize;
            }
            SkipIfEqualAddress(n1, nn) => {
                self.program_counter += if n1 == nn { 4 } else { 2 };
            }
            SkipIfNotEqualAddress(n1, nn) => {
                self.program_counter += if n1 != nn { 4 } else { 2 };
            }
            SkipIfEqualRegister(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
                let registers_are_equal =
                    self.register.v[x as usize] == self.register.v[y as usize];
                self.program_counter += if registers_are_equal { 4 } else { 2 };
            }
            SetRegister(x, nn) => {
                let x = x >> 8;
                self.register.v[x as usize] = nn as u8;
                self.program_counter += 2;
            }
            AddAddressToRegister(x, nn) => {
                let x = x >> 8;
                self.register.v[x as usize] += nn as u8;
                self.program_counter += 2;
            }
            Assign(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
                self.register.v[x as usize] = self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignOr(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
                self.register.v[x as usize] =
                    self.register.v[x as usize] | self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignAnd(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
                self.register.v[x as usize] =
                    self.register.v[x as usize] & self.register.v[y as usize];
                self.program_counter += 2;
            }
            AssignXor(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
                self.register.v[x as usize] =
                    self.register.v[x as usize] ^ self.register.v[y as usize];
                self.program_counter += 2;
            }
            Add(x, y) => {
                // Opcode 0x8XY4
                // Add VY to VX, set carry flag if overflow

                let x = x >> 8; // Convert e.g. 0x0300 to 0x03, so V3
                let y = y >> 4; // Convert e.g. 0x0050 to 0x05, so V5

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
                let x = x >> 8;
                let y = y >> 4;
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
            LeastSigStoreAndShift(x, n2) => {
                // Stores the least significant bit of VX in VF and then shifts VX to the right by 1
                let x = x >> 8;
                // Mask out everything but the least significant bit
                self.register.v[0xF] = self.register.v[x as usize] & 0x1;
                self.register.v[x as usize] >>= 1;
                self.program_counter += 2;
            }
            SetSubtract(x, y) => {
                let x = x >> 8;
                let y = y >> 4;
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
            MostSigStoreAndShift(x, y) => {
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
                self.register.v[x as usize] = nn as u8 & random;
                self.program_counter += 2;
            }
            Display(x, y, n) => {
                // Coordinates at which the sprite is drawn
                // let x = pc & 0x0F00;
                // let y = pc & 0x00F0;
                // The height of the sprite
                // let n = pc & 0x000F;

                // For every row
                for yline in 0..n {
                    // Read 8 pixels (represented through 1 byte) from memory starting at I
                    let byte = self.memory[(self.index_register + yline) as usize];
                    // display at x, y
                    // Before writing the byte, read the current value and check if pixels are flipped
                    // If so, set VF to 1, otherwise to 0

                    // Reset VF to 0
                    self.register.v[0xf] = 1;

                    // xline indicates the position in the line
                    for xline in 0..8 {
                        // 0x80 >> xline generates 128, 64, 32, .. which look like..
                        // 0x40 in binary is       0100 0000
                        // E.g. input 0x45 is      0100 0101
                        // through masking we get  0100 0000
                        let bit = byte & (0x80 >> xline);
                        // and we know bit 7 is set if the value is not decimal(0)
                        // because then the mask would've eliminated all 1s
                        if bit != 0 {
                            let index = (x + xline + (y + yline) * 64) as usize;
                            // Only if the pixel is turned off, set VF = 1
                            if self.gfx[index] == 1 {
                                self.register.v[0xf] = 1;
                            }
                            // Bit was set, so xor the current value
                            self.gfx[index] ^= 1;
                        }
                    }
                }
                self.program_counter += 2;
            }
            SkipIfKeyPressed(x) => {
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                self.program_counter += if self.register.v[x as usize] == 1 {
                    4
                } else {
                    2
                }
            }
            SkipIfNotKeyPressed(x) => {
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                self.program_counter += if self.register.v[x as usize] != 1 {
                    4
                } else {
                    2
                }
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
            UNKNOWN(n1, n2, n3, n4) => println!("Unkown Instruction {}{}{}{}", n1, n2, n3, n4),
            _ => {}
        }
    }
}

// Every group of 5 bytes represent the corresponding character from 0 to F
// if drawn in binary, row by row
static FONTSET: [u8; 80] = 
[0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];
