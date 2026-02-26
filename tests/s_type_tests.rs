use riscv_emulator_rust::RiscvCpu;

/// Encode an S-type instruction (SB/SH/SW).
///
/// imm: 12-bit signed immediate
/// rs2: source register 2 (value to store)
/// rs1: source register 1 (base address)
/// funct3: instruction funct3
/// opcode: 0x23 for store instructions
fn encode_stype(imm: i32, rs2: u8, rs1: u8, funct3: u8) -> u32 {
    let imm11_5 = ((imm >> 5) & 0x7F) as u32;
    let imm4_0 = (imm & 0x1F) as u32;

    (imm11_5 << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | (imm4_0 << 7)
        | 0x23
}

mod sb {
    use super::*;

    #[test]
    fn test_sb_positive_offset() {
        let mut cpu = RiscvCpu::new();
        // base address
        cpu.regs[1] = 0x100;
        // value to store
        cpu.regs[2] = 0x12345678;

        // sb x2, 4(x1) -> store byte 0x78 at 0x104
        let instruction = encode_stype(4, 2, 1, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x104], 0x78);
        assert_eq!(cpu.bus[0x105], 0x00, "Should only write 1 byte");
    }

    #[test]
    fn test_sb_negative_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xFF;

        // sb x2, -4(x1) -> store byte 0xFF at 0xFC
        let instruction = encode_stype(-4, 2, 1, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0xFC], 0xFF);
    }

    #[test]
    fn test_sb_zero_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x200;
        cpu.regs[2] = 0xAB;

        // sb x2, 0(x1) -> store byte 0xAB at 0x200
        let instruction = encode_stype(0, 2, 1, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x200], 0xAB);
    }

    #[test]
    fn test_sb_stores_only_low_byte() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xDEADBEEF;

        // sb should only store the lowest byte (0xEF)
        let instruction = encode_stype(0, 2, 1, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0xEF, "Only low byte should be stored");
        assert_eq!(cpu.bus[0x101], 0x00, "Adjacent byte must not be touched");
        assert_eq!(cpu.bus[0x102], 0x00, "Adjacent byte must not be touched");
        assert_eq!(cpu.bus[0x103], 0x00, "Adjacent byte must not be touched");
    }

    #[test]
    fn test_sb_zero_value() {
        let mut cpu = RiscvCpu::new();
        cpu.bus[0x100] = 0xFF; // pre-fill
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0x00;

        // sb x2, 0(x1) -> store 0x00, overwriting existing value
        let instruction = encode_stype(0, 2, 1, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0x00);
    }

    #[test]
    fn test_sb_x0_as_base() {
        let mut cpu = RiscvCpu::new();
        // x0 is always 0, so base = 0, imm = 4, addr = 4
        cpu.regs[2] = 0x42;

        let instruction = encode_stype(4, 2, 0, 0b000);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[4], 0x42);
    }
}

mod sh {
    use super::*;

    #[test]
    fn test_sh_positive_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0x12345678;

        // sh x2, 4(x1) -> store halfword 0x5678 at 0x104 (little-endian: 0x78 at 0x104, 0x56 at 0x105)
        let instruction = encode_stype(4, 2, 1, 0b001);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x104], 0x78);
        assert_eq!(cpu.bus[0x105], 0x56);
        assert_eq!(cpu.bus[0x106], 0x00, "Should only write 2 bytes");
    }

    #[test]
    fn test_sh_negative_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xABCD;

        // sh x2, -2(x1) -> store halfword at 0xFE
        let instruction = encode_stype(-2, 2, 1, 0b001);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0xFE], 0xCD);
        assert_eq!(cpu.bus[0xFF], 0xAB);
    }

    #[test]
    fn test_sh_zero_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x200;
        cpu.regs[2] = 0xBEEF;

        // sh x2, 0(x1) -> store halfword at 0x200
        let instruction = encode_stype(0, 2, 1, 0b001);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x200], 0xEF);
        assert_eq!(cpu.bus[0x201], 0xBE);
        assert_eq!(cpu.bus[0x202], 0x00, "3rd byte must not be written");
    }

    #[test]
    fn test_sh_stores_only_low_halfword() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xDEADBEEF;

        // sh should only write the low 16 bits (0xBEEF)
        let instruction = encode_stype(0, 2, 1, 0b001);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0xEF, "Low byte of halfword");
        assert_eq!(cpu.bus[0x101], 0xBE, "High byte of halfword");
        assert_eq!(cpu.bus[0x102], 0x00, "3rd byte must not be touched");
        assert_eq!(cpu.bus[0x103], 0x00, "4th byte must not be touched");
    }

    #[test]
    fn test_sh_all_ones() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xFFFF;

        let instruction = encode_stype(0, 2, 1, 0b001);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0xFF);
        assert_eq!(cpu.bus[0x101], 0xFF);
        assert_eq!(cpu.bus[0x102], 0x00, "Must not touch byte beyond halfword");
    }
}

mod sw {
    use super::*;

    #[test]
    fn test_sw_positive_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0x12345678;

        // sw x2, 4(x1) -> store word 0x12345678 at 0x104 (little-endian: 0x78, 0x56, 0x34, 0x12)
        let instruction = encode_stype(4, 2, 1, 0b010);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x104], 0x78);
        assert_eq!(cpu.bus[0x105], 0x56);
        assert_eq!(cpu.bus[0x106], 0x34);
        assert_eq!(cpu.bus[0x107], 0x12);
        assert_eq!(cpu.bus[0x108], 0x00, "Should only write 4 bytes");
    }

    #[test]
    fn test_sw_negative_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xDEADBEEF;

        // sw x2, -4(x1) -> store word 0xDEADBEEF at 0xFC
        let instruction = encode_stype(-4, 2, 1, 0b010);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0xFC], 0xEF);
        assert_eq!(cpu.bus[0xFD], 0xBE);
        assert_eq!(cpu.bus[0xFE], 0xAD);
        assert_eq!(cpu.bus[0xFF], 0xDE);
    }

    #[test]
    fn test_sw_zero_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x200;
        cpu.regs[2] = 0xCAFEBABE;

        // sw x2, 0(x1) -> store at 0x200
        let instruction = encode_stype(0, 2, 1, 0b010);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x200], 0xBE);
        assert_eq!(cpu.bus[0x201], 0xBA);
        assert_eq!(cpu.bus[0x202], 0xFE);
        assert_eq!(cpu.bus[0x203], 0xCA);
        assert_eq!(cpu.bus[0x204], 0x00, "Byte beyond word must not be touched");
    }

    #[test]
    fn test_sw_all_zeros() {
        let mut cpu = RiscvCpu::new();
        // Pre-fill 4 bytes
        cpu.bus[0x100] = 0xAA;
        cpu.bus[0x101] = 0xBB;
        cpu.bus[0x102] = 0xCC;
        cpu.bus[0x103] = 0xDD;
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0x0000_0000;

        // sw x2, 0(x1) -> overwrite with zeros
        let instruction = encode_stype(0, 2, 1, 0b010);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0x00);
        assert_eq!(cpu.bus[0x101], 0x00);
        assert_eq!(cpu.bus[0x102], 0x00);
        assert_eq!(cpu.bus[0x103], 0x00);
    }

    #[test]
    fn test_sw_all_ones() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x100;
        cpu.regs[2] = 0xFFFF_FFFF;

        let instruction = encode_stype(0, 2, 1, 0b010);
        cpu.handle_store(instruction);

        assert_eq!(cpu.bus[0x100], 0xFF);
        assert_eq!(cpu.bus[0x101], 0xFF);
        assert_eq!(cpu.bus[0x102], 0xFF);
        assert_eq!(cpu.bus[0x103], 0xFF);
        assert_eq!(cpu.bus[0x104], 0x00, "Byte beyond word must not be touched");
    }
}
