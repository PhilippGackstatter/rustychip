fn main() {
    let mut cpu = CPU::new();

    loop {

        // emulate a cycle
        cpu.emulate_cycle();
        // Draw if the flag is set

        // Store keypress state

    }
}

// Memory Map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

struct CPU {
    // Memory
    memory: Vec<u8>,
    // Can have values between 0x000 and 0xFFF
    index_register: u16,
    // Can have values between 0x000 and 0xFFF
    program_counter: u16,
    // 64 x 32 pixel video array, black (0) or white (1)
    gfx: Vec<u8>,
    register: Register,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    stack_pointer: u16,
    keypad: Vec<u8>,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            memory: Vec::with_capacity(4096), // 0xfff + 1 = 0x1000
            gfx: Vec::with_capacity(64 * 32),
            keypad: Vec::with_capacity(16),
            stack: Vec::with_capacity(16),
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            index_register: 0,
            program_counter: 0x200, // Start execution from this address
            register: Register::new(),
        }
    }
    pub fn emulate_cycle(&mut self) {
        let opc = self.fetch();
        let opC = self.decode(opc);
        self.emulate(opC);
    }
    fn fetch(&self) -> u16 {
        // Fetch 2 bytes to get the 16 bit opcode
        // Convert the u8s to u16s, so we can safely shift them by 8 bits
        let opcode1 = self.memory[self.program_counter as usize] as u16;
        let opcode2 = self.memory[(self.program_counter + 1) as usize] as u16;
        opcode1 << 8 | opcode2
    }
    fn decode(&self, opcode: u16) -> Opcode {
        // Map the u16 to the actual Opcode
        // Just look at the first 4 bits, to determine the opcode
        match opcode & 0xF000 {
            pc @ 0xA000 => Opcode::ANNN(pc),
            // Two Opcodes begin with 8 bits rather than 4
            opcode @ 0x0000 => {
                match opcode {
                    0x00EE => Opcode::CLRSCRN,
                    0x00E0 => Opcode::RTRN,
                    pc @ _ => Opcode::UNKNOWN(pc)
                }
            },
            pc @ _ => Opcode::UNKNOWN(pc),
        }
    }
    fn emulate(&mut self, opcode: Opcode) {
        use self::Opcode;
        match opcode {
            Opcode::ANNN(pc) => {
                // Set I to NNN
                self.index_register = pc & 0x0FFF;
                self.program_counter += 2;
            },
            Opcode::CLRSCRN => {
                // Clear Screen
                self.program_counter += 2;
            }
            _ => {}
        }
    }
}

// 15 1-byte general purpose registers
// The 16th register is used  for the ‘carry flag’
struct Register {
    v: Vec<u8>, // V0: char,
                // V1: char,
                // V2: char,
                // V3: char,
                // V4: char,
                // V5: char,
                // V6: char,
                // V7: char,
                // V8: char,
                // V9: char,
                // VA: char,
                // VB: char,
                // VC: char,
                // VD: char,
                // VE: char,
                // VF: char
}

impl Register {
    fn new() -> Register {
        Register {
            v: Vec::with_capacity(16),
        }
    }
}

enum Opcode {
    ANNN(u16), // MEM I = NNN Sets I to the address NNN.
    CLRSCRN,
    RTRN,
    UNKNOWN(u16),
}
