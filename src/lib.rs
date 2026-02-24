pub struct RiscvCpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub bus: Vec<u8>,
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

        println!("PC: {:#010x} | Instruction: {:#010x}", self.pc, instruction);

        let opcode = instruction & 0x7f;
        
        match opcode {
            0x13 => {
                self.handle_itype(instruction);
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

    fn write_reg(&mut self, reg: u32, value: u32) {
        if reg != 0 {
            self.regs[reg as usize] = value;
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
            0x0 => {
                match funct7 {
                    0x00 => rs1_value.wrapping_add(rs2_value),
                    0x20 => rs1_value.wrapping_sub(rs2_value),
                    _ => panic!("Invalid funct7 {:#x} for funct3 0x0", funct7),
                }
            }
            0x4 => rs1_value ^ rs2_value,
            0x6 => rs1_value | rs2_value,
            0x7 => rs1_value & rs2_value,
            0x1 => rs1_value << (rs2_value & 0x1F),
            0x5 => {
                match funct7 {
                    0x00 => {
                        rs1_value >> (rs2_value & 0x1F)
                    }
                    0x20 => {
                        ((rs1_value as i32) >> (rs2_value & 0x1F)) as u32
                    }
                    _ => {
                        panic!("Invalid funct7 {:#x} for funct3 {:#x}",funct3, funct7);
                    }
                }
            }
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

        println!("rd: {:#07x}", rd);
        println!("Fuct3: {:#03x}", funct3);
        println!("rs: {:#07x}", rs);
        println!("Sign Extended Imm: {:#034x}", imm);

        let rd_value = match funct3 {
            0x0 => {
                let result = (rs_value as i32 + imm) as u32;
                println!("rd: {} = rs: {} + imm: {} ", result, rs_value, imm);
                result
            }
            0x4 => {
                let result = rs_value ^ (imm as u32);
                println!("rd: {} = rs: {} ^ imm: {} ", result, rs_value, imm);
                result
            }
            0x6 => {
                let result = rs_value | (imm as u32);
                println!("rd: {} = rs: {} | imm: {} ", result, rs_value, imm);
                result
            }
            0x7 => {
                let result = rs_value & (imm as u32);
                println!("rd: {} = rs: {} & imm: {} ", result, rs_value, imm);
                result
            }
            0x1 => {
                let shift = imm & 0x1F;
                let result = rs_value << shift;
                println!("rd: {:#010x} = rs: {:#010x} << imm[0:4]: {} ", result, rs_value, shift);
                result
            }
            0x5 => {
                let funct7 = (instruction >> 25) & 0x7F;
                let shamt = imm & 0x1F;

                match funct7 {
                    0x00 => {
                        println!("rd: {:#010x} = rs: {:#010x} >> imm[0:4]: {} ", rs_value >> shamt, rs_value, shamt);
                        rs_value >> shamt
                    }
                    0x20 => {
                        println!("rd: {:#010x} = rs: {:#010x} >> imm[0:4]: {} ", (rs_value as i32 >> shamt) as u32, rs_value, shamt);
                        (rs_value as i32 >> shamt) as u32
                    }
                    _ => {
                        panic!("Invalid funct7 {:#x} for funct3 0x5", funct7);
                    }
                }
            }
            0x2 => {
                if (rs_value as i32) < imm {
                    1 as u32
                } else {
                    0 as u32
                }
            }
            0x3 => {
                if rs_value < (imm as u32) {
                    1 as u32
                } else  {
                    0 as u32
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

}
