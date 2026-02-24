struct RiscvCpu {
    regs: [u32; 32],
    pc: u32,
    bus: Vec<u8>,
}

impl RiscvCpu {
    fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            bus: vec![0; 1024],
        }
    }

    fn step(&mut self) {
        let instruction: u32 = self.fetch();

        println!("PC: {:#010x} | Instruction: {:#010x}", self.pc, instruction);

        let opcode = instruction & 0x7f;
        
        match opcode {
            0x13 => {
                println!("Do something");
                let funct3 = (instruction >> 12)
                let imm = (instruction >> 20) & 0xfff;
                println!("12-bit Imm: {:#05x}", imm);
            }
            _ =>  println!("don't have this yet"),
        }

        self.pc += 4;
    }

    fn fetch(&self) -> u32 {
        let addr = self.pc as usize;
        let bytes = &self.bus[addr..addr+4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

fn main() {
    let mut cpu = RiscvCpu::new();

    let test_instr: u32 = 0x00100513;
    let bytes = test_instr.to_le_bytes();
    cpu.bus[0..4].copy_from_slice(&bytes);

    cpu.step();
    cpu.step();
}