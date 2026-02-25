use riscv_emulator_rust::RiscvCpu;

/// Encode a B-type instruction (BEQ/BNE/BLT/BGE/BLTU/BGEU).
///
/// imm: 13-bit signed immediate (must be a multiple of 2)
/// rs1: source register 1
/// rs2: source register 2
/// funct3: instruction funct3
/// opcode: 0x63 for branch instructions
fn encode_btype(imm: i32, rs1: u8, rs2: u8, funct3: u8) -> u32 {
    let imm = imm as u32;
    let b12 = (imm >> 12) & 0x1;
    let b11 = (imm >> 11) & 0x1;
    let b10_5 = (imm >> 5) & 0x3F;
    let b4_1 = (imm >> 1) & 0xF;
    let opcode = 0x63u32;

    (b12 << 31)
        | (b10_5 << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | (b4_1 << 8)
        | (b11 << 7)
        | opcode
}

mod beq {
    use super::*;

    #[test]
    fn test_beq_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 10;
        cpu.pc = 0x100;

        // beq x1, x2, 8 (PC = 0x100 + 8 = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b000);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 20;
        cpu.pc = 0x100;

        // beq x1, x2, 8 (Not taken, PC = 0x100 + 4 = 0x104)
        let instruction = encode_btype(8, 1, 2, 0b000);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x104);
    }
}

mod bne {
    use super::*;

    #[test]
    fn test_bne_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 20;
        cpu.pc = 0x100;

        // bne x1, x2, 8 (Taken, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b001);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_bne_not_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 10;
        cpu.pc = 0x100;

        // bne x1, x2, 8 (Not taken, PC = 0x104)
        let instruction = encode_btype(8, 1, 2, 0b001);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x104);
    }
}

mod blt {
    use super::*;

    #[test]
    fn test_blt_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;
        cpu.regs[2] = 10;
        cpu.pc = 0x100;

        // blt x1, x2, 8 (Taken: 5 < 10, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b100);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_blt_signed_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = -10i32 as u32;
        cpu.regs[2] = 5;
        cpu.pc = 0x100;

        // blt x1, x2, 8 (Taken: -10 < 5, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b100);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }
}

mod bge {
    use super::*;

    #[test]
    fn test_bge_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 5;
        cpu.pc = 0x100;

        // bge x1, x2, 8 (Taken: 10 >= 5, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b101);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_bge_equal_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 10;
        cpu.pc = 0x100;

        // bge x1, x2, -4 (Taken: 10 >= 10, PC = 0x0FC)
        let instruction = encode_btype(-4, 1, 2, 0b101);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x0FC);
    }
}

mod bltu {
    use super::*;

    #[test]
    fn test_bltu_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;
        cpu.regs[2] = 0xFFFF_FFFF; // large unsigned
        cpu.pc = 0x100;

        // bltu x1, x2, 8 (Taken: 5 < big, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b110);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_bltu_not_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 5;
        cpu.pc = 0x100;

        // bltu x1, x2, 8 (Not taken: big < 5 is false, PC = 0x104)
        let instruction = encode_btype(8, 1, 2, 0b110);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x104);
    }
}

mod bgeu {
    use super::*;

    #[test]
    fn test_bgeu_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 5;
        cpu.pc = 0x100;

        // bgeu x1, x2, 8 (Taken: big >= 5, PC = 0x108)
        let instruction = encode_btype(8, 1, 2, 0b111);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x108);
    }

    #[test]
    fn test_bgeu_equal_taken() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xABC;
        cpu.regs[2] = 0xABC;
        cpu.pc = 0x100;

        // bgeu x1, x2, 12 (Taken: 0xABC >= 0xABC, PC = 0x10C)
        let instruction = encode_btype(12, 1, 2, 0b111);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_btype(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.pc, 0x10C);
    }
}
