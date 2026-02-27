pub struct RiscvCpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub bus: Vec<u8>,
}

#[derive(Copy, Clone)]
pub enum MemSize {
    Byte,
    Half,
    Word,
}

impl RiscvCpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            bus: vec![0; 1024],
        }
    }

    pub fn step(&mut self) {
        let instruction: u32 = self.fetch();

        let mut next_pc = self.pc + 4;

        println!("PC: {:#010x} | Instruction: {:#010x}", self.pc, instruction);

        let opcode = instruction & 0x7f;

        match opcode {
            0x33 => self.handle_rtype(instruction),
            0x13 => self.handle_itype(instruction),
            0x03 => self.handle_load(instruction),
            0x23 => self.handle_store(instruction),
            0x63 => self.handle_btype(instruction, &mut next_pc),
            0x6F => self.handle_jal(instruction, &mut next_pc),
            0x67 => self.handle_jalr(instruction, &mut next_pc),
            0x37 => self.handle_lui(instruction),
            0x17 => self.handle_auipc(instruction),
            _ => println!("don't have this yet"),
        }

        self.pc = next_pc;
    }

    pub fn load(&self, addr: u32, size: MemSize, signed: bool) -> u32 {
        let raw = self.read_raw(addr, size);

        if !signed {
            return raw;
        }

        match size {
            MemSize::Byte => (raw as i8 as i32) as u32,
            MemSize::Half => (raw as i16 as i32) as u32,
            MemSize::Word => raw,
        }
    }

    pub fn store(&mut self, addr: u32, size: MemSize, value: u32) {
        let a = addr as usize;

        match size {
            MemSize::Byte => self.bus[a] = (value & 0xFF) as u8,
            MemSize::Half => {
                self.bus[a] = (value & 0xFF) as u8;
                self.bus[a + 1] = ((value >> 8) & 0xFF) as u8;
            }
            MemSize::Word => {
                self.bus[a] = (value & 0xFF) as u8;
                self.bus[a + 1] = ((value >> 8) & 0xFF) as u8;
                self.bus[a + 2] = ((value >> 16) & 0xFF) as u8;
                self.bus[a + 3] = ((value >> 24) & 0xFF) as u8;
            }
        }
    }

    pub fn handle_rtype(&mut self, instruction: u32) {
        let rd = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = (instruction >> 15) & 0x1F;
        let rs2 = (instruction >> 20) & 0x1F;
        let funct7 = (instruction >> 25) & 0x7f;

        let rs1_value = self.regs[rs1 as usize];
        let rs2_value = self.regs[rs2 as usize];

        let rd_value = match funct3 {
            0x0 => match funct7 {
                0x00 => rs1_value.wrapping_add(rs2_value),
                0x20 => rs1_value.wrapping_sub(rs2_value),
                _ => panic!("Invalid funct7 {:#x} for funct3 0x0", funct7),
            },
            0x4 => rs1_value ^ rs2_value,
            0x6 => rs1_value | rs2_value,
            0x7 => rs1_value & rs2_value,
            0x1 => rs1_value << (rs2_value & 0x1F),
            0x5 => match funct7 {
                0x00 => rs1_value >> (rs2_value & 0x1F),
                0x20 => ((rs1_value as i32) >> (rs2_value & 0x1F)) as u32,
                _ => {
                    panic!("Invalid funct7 {:#x} for funct3 {:#x}", funct3, funct7);
                }
            },
            0x2 => {
                if (rs1_value as i32) < (rs2_value as i32) {
                    1
                } else {
                    0
                }
            }
            0x3 => {
                if rs1_value < rs2_value {
                    1
                } else {
                    0
                }
            }
            _ => {
                panic!("Invalid funct3 {:#x}", funct3)
            }
        };

        self.write_reg(rd, rd_value);
    }

    pub fn handle_itype(&mut self, instruction: u32) {
        let rd = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs = (instruction >> 15) & 0x1F;
        let imm = (instruction as i32) >> 20;
        let rs_value = self.regs[rs as usize];

        let rd_value = match funct3 {
            0x0 => (rs_value as i32 + imm) as u32,
            0x4 => rs_value ^ (imm as u32),
            0x6 => rs_value | (imm as u32),
            0x7 => rs_value & (imm as u32),
            0x1 => rs_value << (imm & 0x1F),
            0x5 => {
                let funct7 = (instruction >> 25) & 0x7F;
                let shamt = imm & 0x1F;

                match funct7 {
                    0x00 => {
                        println!(
                            "rd: {:#010x} = rs: {:#010x} >> imm[0:4]: {} ",
                            rs_value >> shamt,
                            rs_value,
                            shamt
                        );
                        rs_value >> shamt
                    }
                    0x20 => {
                        println!(
                            "rd: {:#010x} = rs: {:#010x} >> imm[0:4]: {} ",
                            (rs_value as i32 >> shamt) as u32,
                            rs_value,
                            shamt
                        );
                        (rs_value as i32 >> shamt) as u32
                    }
                    _ => {
                        panic!("Invalid funct7 {:#x} for funct3 0x5", funct7);
                    }
                }
            }
            0x2 => {
                if (rs_value as i32) < imm {
                    1_u32
                } else {
                    0_u32
                }
            }
            0x3 => {
                if rs_value < (imm as u32) {
                    1_u32
                } else {
                    0_u32
                }
            }
            _ => {
                panic!("Unknown funct3: {:#x}", funct3);
            }
        };

        if rd != 0 {
            self.write_reg(rd, rd_value);
        }
    }

    pub fn handle_load(&mut self, instruction: u32) {
        let rd = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs = (instruction >> 15) & 0x1F;
        let imm = (instruction as i32) >> 20;
        let rs_value = self.regs[rs as usize] as i32;
        let addr = rs_value.wrapping_add(imm) as u32;

        let rd_value = match funct3 {
            0x0 => self.load(addr, MemSize::Byte, true),
            0x1 => self.load(addr, MemSize::Half, true),
            0x2 => self.load(addr, MemSize::Word, true),
            0x4 => self.load(addr, MemSize::Byte, false),
            0x5 => self.load(addr, MemSize::Half, false),
            _ => panic!("Unknown funct3: {:#x}", funct3),
        };

        self.write_reg(rd, rd_value);
    }

    pub fn handle_store(&mut self, instruction: u32) {
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = (instruction >> 15) & 0x1F;
        let rs2 = (instruction >> 20) & 0x1F;

        let i4_0 = (instruction >> 7) & 0x1F;
        let i11_5 = (instruction >> 25) & 0x7F;

        let imm_u: u32 = (i11_5 << 5) | i4_0;
        let imm = ((imm_u << 20) as i32) >> 20;

        let rs1_value = self.regs[rs1 as usize];
        let rs2_value = self.regs[rs2 as usize];

        let addr = (rs1_value as i32).wrapping_add(imm) as u32;

        match funct3 {
            0x0 => self.store(addr, MemSize::Byte, rs2_value),
            0x1 => self.store(addr, MemSize::Half, rs2_value),
            0x2 => self.store(addr, MemSize::Word, rs2_value),
            _ => panic!("Unknown S-type funct3: {:#x}", funct3),
        }
    }

    pub fn handle_btype(&mut self, instruction: u32, next_pc: &mut u32) {
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = (instruction >> 15) & 0x1F;
        let rs2 = (instruction >> 20) & 0x1F;

        let i12 = (instruction >> 31) & 0x1;
        let i11 = (instruction >> 7) & 0x1;
        let i10_5 = (instruction >> 25) & 0x3F;
        let i4_1 = (instruction >> 8) & 0xF;

        let imm_u32 = (i12 << 12) | (i11 << 11) | (i10_5 << 5) | (i4_1 << 1);
        let imm = ((imm_u32 << 19) as i32) >> 19;

        let rs1_value = self.regs[rs1 as usize];
        let rs2_value = self.regs[rs2 as usize];

        let should_branch = match funct3 {
            0x0 => rs1_value == rs2_value,
            0x1 => rs1_value != rs2_value,
            0x4 => (rs1_value as i32) < (rs2_value as i32),
            0x5 => (rs1_value as i32) >= (rs2_value as i32),
            0x6 => rs1_value < rs2_value,
            0x7 => rs1_value >= rs2_value,
            _ => panic!("Unknown B-type funct3: {:#x}", funct3),
        };

        if should_branch {
            *next_pc = (self.pc as i32).wrapping_add(imm) as u32;
        }
    }

    pub fn handle_jal(&mut self, instruction: u32, next_pc: &mut u32) {
        let rd = (instruction >> 7) & 0x1F;

        let i19_12 = (instruction >> 12) & 0xFF;
        let i11 = (instruction >> 20) & 0x1;
        let i10_1 = (instruction >> 21) & 0x3FF;
        let i20 = (instruction >> 31) & 0x1;

        let imm_u32 = (i20 << 20) | (i19_12 << 12) | (i11 << 11) | (i10_1 << 1);
        let imm = ((imm_u32 << 11) as i32) >> 11;

        let rd_value = self.pc + 4;
        self.write_reg(rd, rd_value);
        *next_pc = (self.pc as i32).wrapping_add(imm) as u32;
    }

    pub fn handle_jalr(&mut self, instruction: u32, next_pc: &mut u32) {
        let rd = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs = (instruction >> 15) & 0x1F;
        let imm = (instruction as i32) >> 20;
        let rs_value = self.regs[rs as usize];

        let rd_value = self.pc + 4;

        match funct3 {
            0x0 => *next_pc = (rs_value as i32).wrapping_add(imm) as u32,
            _ => panic!("Unknown JALR funct3 {:#x}", funct3),
        }

        self.write_reg(rd, rd_value);
    }

    pub fn handle_lui(&mut self, instruction: u32) {
        let rd = (instruction >> 7) & 0x1F;
        let imm = (instruction >> 12) & 0xFFFFF;

        let rd_value = imm << 12;
        self.write_reg(rd, rd_value);
    }

    pub fn handle_auipc(&mut self, instruction: u32) {
        let rd = (instruction >> 7) & 0x1F;
        let imm = (instruction >> 12) & 0xFFFFF;

        let offset = (imm << 12) as i32;

        let rd_value = (self.pc as i32).wrapping_add(offset) as u32;
        self.write_reg(rd, rd_value);
    }

    fn fetch(&self) -> u32 {
        let addr = self.pc as usize;
        let bytes = &self.bus[addr..addr + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn write_reg(&mut self, reg: u32, value: u32) {
        if reg != 0 {
            self.regs[reg as usize] = value;
        }
    }

    fn read_raw(&self, addr: u32, size: MemSize) -> u32 {
        let a = addr as usize;
        match size {
            MemSize::Byte => self.bus[a] as u32,
            MemSize::Half => (self.bus[a] as u32) | ((self.bus[a + 1] as u32) << 8),
            MemSize::Word => {
                (self.bus[a] as u32)
                    | ((self.bus[a + 1] as u32) << 8)
                    | ((self.bus[a + 2] as u32) << 16)
                    | ((self.bus[a + 3] as u32) << 24)
            }
        }
    }
}
