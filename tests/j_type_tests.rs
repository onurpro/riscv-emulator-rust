use riscv_emulator_rust::RiscvCpu;

// ── Instruction encoders ──────────────────────────────────────────────────────

/// Encode a J-type (JAL) instruction.
///
/// imm: 21-bit signed immediate (must be a multiple of 2), PC-relative offset
/// rd:  destination register (receives PC+4)
/// opcode: 0x6F for JAL
fn encode_jal(imm: i32, rd: u8) -> u32 {
    let imm = imm as u32;
    let i20 = (imm >> 20) & 0x1;
    let i19_12 = (imm >> 12) & 0xFF;
    let i11 = (imm >> 11) & 0x1;
    let i10_1 = (imm >> 1) & 0x3FF;

    (i20 << 31) | (i10_1 << 21) | (i11 << 20) | (i19_12 << 12) | ((rd as u32) << 7) | 0x6F
}

/// Encode a JALR instruction.
///
/// imm:   12-bit signed immediate offset added to rs1
/// rs1:   base register
/// rd:    destination register (receives PC+4)
/// funct3 is always 0x0 for JALR; opcode is 0x67.
fn encode_jalr(imm: i32, rs1: u8, rd: u8) -> u32 {
    let imm12 = (imm & 0xFFF) as u32;
    (imm12 << 20) | ((rs1 as u32) << 15) | (0b000u32 << 12) | ((rd as u32) << 7) | 0x67
}

/// Encode a LUI instruction.
///
/// imm: upper 20 bits placed in bits [31:12] of rd.  Pass the raw 20-bit value
///      (before shifting) – the encoder shifts it for you.
fn encode_lui(imm: u32, rd: u8) -> u32 {
    ((imm & 0xFFFFF) << 12) | ((rd as u32) << 7) | 0x37
}

/// Encode an AUIPC instruction.
///
/// imm: upper 20 bits added to PC. Same convention as encode_lui.
fn encode_auipc(imm: u32, rd: u8) -> u32 {
    ((imm & 0xFFFFF) << 12) | ((rd as u32) << 7) | 0x17
}

// ── JAL ───────────────────────────────────────────────────────────────────────

mod jal {
    use super::*;

    #[test]
    fn test_jal_positive_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;

        // jal x1, +8  →  rd = PC+4 = 0x104, next_pc = 0x100 + 8 = 0x108
        let instruction = encode_jal(8, 1);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[1], 0x104, "x1 (return address) should be old PC+4");
        assert_eq!(cpu.pc, 0x108, "PC should jump to PC+8");
    }

    #[test]
    fn test_jal_negative_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;

        // jal x1, -8  →  rd = 0x104, next_pc = 0x100 + (-8) = 0x0F8
        let instruction = encode_jal(-8, 1);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[1], 0x104, "x1 should be old PC+4");
        assert_eq!(cpu.pc, 0x0F8, "PC should jump backward to 0x0F8");
    }

    #[test]
    fn test_jal_large_positive_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x000;

        // jal x2, +0x100  →  rd = 0x4, next_pc = 0x100
        let instruction = encode_jal(0x100, 2);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[2], 0x4, "x2 should be 0x4 (return address)");
        assert_eq!(cpu.pc, 0x100, "PC should be 0x100");
    }

    #[test]
    fn test_jal_rd_zero_discards_return_address() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x200;

        // jal x0, +4  →  x0 must remain 0 (write to x0 is a no-op)
        let instruction = encode_jal(4, 0);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[0], 0, "x0 must always be 0");
        assert_eq!(cpu.pc, 0x204, "PC should still jump correctly");
    }

    #[test]
    fn test_jal_return_address_is_pc_plus_4() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x3FC;

        // Whatever the offset, rd must always be old_pc + 4
        let instruction = encode_jal(16, 5);
        let old_pc = cpu.pc;
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[5], old_pc + 4, "return address is always PC+4");
    }

    #[test]
    fn test_jal_pc_zero_forward_jump() {
        let mut cpu = RiscvCpu::new();
        // pc = 0 (default)

        // jal x3, +20  →  rd = 4, next_pc = 20
        let instruction = encode_jal(20, 3);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jal(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[3], 4, "x3 = return address = 4");
        assert_eq!(cpu.pc, 20, "PC = 0 + 20 = 20");
    }
}

// ── JALR ──────────────────────────────────────────────────────────────────────

mod jalr {
    use super::*;

    #[test]
    fn test_jalr_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;
        cpu.regs[1] = 0x200; // base address

        // jalr x2, x1, 0  →  rd = 0x104, next_pc = 0x200 + 0 = 0x200
        let instruction = encode_jalr(0, 1, 2);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[2], 0x104, "x2 (return address) should be old PC+4");
        assert_eq!(cpu.pc, 0x200, "PC should jump to rs1");
    }

    #[test]
    fn test_jalr_positive_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;
        cpu.regs[1] = 0x200;

        // jalr x2, x1, 8  →  rd = 0x104, next_pc = 0x200 + 8 = 0x208
        let instruction = encode_jalr(8, 1, 2);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[2], 0x104, "return address must be 0x104");
        assert_eq!(cpu.pc, 0x208, "PC = rs1 + 8 = 0x208");
    }

    #[test]
    fn test_jalr_negative_offset() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;
        cpu.regs[1] = 0x200;

        // jalr x2, x1, -8  →  rd = 0x104, next_pc = 0x200 + (-8) = 0x1F8
        let instruction = encode_jalr(-8, 1, 2);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[2], 0x104, "return address must be 0x104");
        assert_eq!(cpu.pc, 0x1F8, "PC = 0x200 - 8 = 0x1F8");
    }

    #[test]
    fn test_jalr_rd_zero_discards_return_address() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;
        cpu.regs[1] = 0x400;

        // jalr x0, x1, 0  →  x0 stays 0, next_pc = 0x400
        let instruction = encode_jalr(0, 1, 0);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[0], 0, "x0 must always be 0");
        assert_eq!(cpu.pc, 0x400, "PC should still jump to rs1");
    }

    #[test]
    fn test_jalr_rs1_is_rd() {
        // When rd == rs1, the spec says rd = PC+4 and the jump uses the OLD rs1 value.
        // Our implementation reads rs1 before writing rd, so this should work correctly.
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;
        cpu.regs[1] = 0x300; // rs1 = x1

        // jalr x1, x1, 0 → rd/rs1 are the same register (x1)
        // Expected: PC jumps to old rs1 value (0x300), x1 = old PC+4 = 0x104
        let instruction = encode_jalr(0, 1, 1);
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        // The implementation saves rd_value = pc+4 before overwriting, so next_pc
        // was captured from old rs1 before write_reg is called.
        assert_eq!(cpu.regs[1], 0x104, "x1 should hold return address");
        assert_eq!(cpu.pc, 0x300, "PC jumps to original rs1 (0x300)");
    }

    #[test]
    fn test_jalr_return_address_is_pc_plus_4() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x5FC;
        cpu.regs[3] = 0x800;

        let instruction = encode_jalr(4, 3, 4);
        let old_pc = cpu.pc;
        let mut next_pc = cpu.pc + 4;
        cpu.handle_jalr(instruction, &mut next_pc);
        cpu.pc = next_pc;

        assert_eq!(cpu.regs[4], old_pc + 4, "return address is always PC+4");
        assert_eq!(cpu.pc, 0x804, "PC = 0x800 + 4");
    }
}

// ── LUI ───────────────────────────────────────────────────────────────────────

mod lui {
    use super::*;

    #[test]
    fn test_lui_basic() {
        let mut cpu = RiscvCpu::new();

        // lui x1, 1  →  x1 = 1 << 12 = 0x1000
        let instruction = encode_lui(1, 1);
        cpu.handle_lui(instruction);

        assert_eq!(cpu.regs[1], 0x1000, "x1 = 1 shifted left 12 bits");
    }

    #[test]
    fn test_lui_max_imm() {
        let mut cpu = RiscvCpu::new();

        // lui x1, 0xFFFFF  →  x1 = 0xFFFFF000
        let instruction = encode_lui(0xFFFFF, 1);
        cpu.handle_lui(instruction);

        assert_eq!(cpu.regs[1], 0xFFFFF000, "x1 should be 0xFFFFF000");
    }

    #[test]
    fn test_lui_zero_imm() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[2] = 0xDEAD; // pre-set to something non-zero

        // lui x2, 0  →  x2 = 0
        let instruction = encode_lui(0, 2);
        cpu.handle_lui(instruction);

        assert_eq!(cpu.regs[2], 0, "LUI with imm=0 clears the register");
    }

    #[test]
    fn test_lui_rd_zero_is_nop() {
        let mut cpu = RiscvCpu::new();

        // lui x0, 0xABCDE  →  x0 stays 0
        let instruction = encode_lui(0xABCDE, 0);
        cpu.handle_lui(instruction);

        assert_eq!(cpu.regs[0], 0, "x0 must always be 0");
    }

    #[test]
    fn test_lui_lower_12_bits_are_zero() {
        let mut cpu = RiscvCpu::new();

        // lui x3, 0x12345  →  x3 = 0x12345000 (lower 12 bits must be 0)
        let instruction = encode_lui(0x12345, 3);
        cpu.handle_lui(instruction);

        assert_eq!(
            cpu.regs[3] & 0xFFF,
            0,
            "lower 12 bits of LUI result must be 0"
        );
        assert_eq!(cpu.regs[3], 0x12345000, "x3 = 0x12345000");
    }

    #[test]
    fn test_lui_multiple_registers() {
        let mut cpu = RiscvCpu::new();

        cpu.handle_lui(encode_lui(0x00001, 1));
        cpu.handle_lui(encode_lui(0x00010, 2));
        cpu.handle_lui(encode_lui(0x00100, 3));

        assert_eq!(cpu.regs[1], 0x0000_1000, "x1 = 0x1000");
        assert_eq!(cpu.regs[2], 0x0001_0000, "x2 = 0x10000");
        assert_eq!(cpu.regs[3], 0x0010_0000, "x3 = 0x100000");
    }
}

// ── AUIPC ─────────────────────────────────────────────────────────────────────

mod auipc {
    use super::*;

    #[test]
    fn test_auipc_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x1000;

        // auipc x1, 1  →  x1 = PC + (1 << 12) = 0x1000 + 0x1000 = 0x2000
        let instruction = encode_auipc(1, 1);
        cpu.handle_auipc(instruction);

        assert_eq!(cpu.regs[1], 0x2000, "x1 = PC + 0x1000");
    }

    #[test]
    fn test_auipc_zero_imm() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x300;

        // auipc x2, 0  →  x2 = PC + 0 = 0x300
        let instruction = encode_auipc(0, 2);
        cpu.handle_auipc(instruction);

        assert_eq!(cpu.regs[2], 0x300, "auipc with imm=0 copies PC into rd");
    }

    #[test]
    fn test_auipc_from_zero_pc() {
        let mut cpu = RiscvCpu::new();
        // PC = 0 (default)

        // auipc x1, 2  →  x1 = 0 + (2 << 12) = 0x2000
        let instruction = encode_auipc(2, 1);
        cpu.handle_auipc(instruction);

        assert_eq!(cpu.regs[1], 0x2000, "x1 = 0 + 0x2000");
    }

    #[test]
    fn test_auipc_max_imm() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x0;

        // auipc x1, 0xFFFFF  →  x1 = 0 + 0xFFFFF000
        let instruction = encode_auipc(0xFFFFF, 1);
        cpu.handle_auipc(instruction);

        assert_eq!(cpu.regs[1], 0xFFFFF000, "x1 = 0xFFFFF000");
    }

    #[test]
    fn test_auipc_wrapping_overflow() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0xFFFF_F000;

        // auipc x1, 1  →  with wrapping: 0xFFFF_F000 + 0x1000 wraps to 0x0000_0000
        let instruction = encode_auipc(1, 1);
        cpu.handle_auipc(instruction);

        let expected = 0xFFFF_F000u32.wrapping_add(0x1000);
        assert_eq!(cpu.regs[1], expected, "auipc should wrap correctly");
    }

    #[test]
    fn test_auipc_rd_zero_is_nop() {
        let mut cpu = RiscvCpu::new();
        cpu.pc = 0x100;

        // auipc x0, 5  →  x0 stays 0
        let instruction = encode_auipc(5, 0);
        cpu.handle_auipc(instruction);

        assert_eq!(cpu.regs[0], 0, "x0 must always be 0");
    }

    #[test]
    fn test_auipc_result_is_pc_relative() {
        let mut cpu = RiscvCpu::new();

        // Run AUIPC at two different PC values and verify both results are PC-relative.
        let imm: u32 = 0x10; // 0x10 << 12 = 0x10000

        cpu.pc = 0x0;
        cpu.handle_auipc(encode_auipc(imm, 1));
        let result_at_0 = cpu.regs[1];

        cpu.pc = 0x200;
        cpu.handle_auipc(encode_auipc(imm, 2));
        let result_at_200 = cpu.regs[2];

        assert_eq!(result_at_0, 0x0000_0000u32.wrapping_add(imm << 12));
        assert_eq!(result_at_200, 0x0000_0200u32.wrapping_add(imm << 12));
        assert_eq!(
            result_at_200.wrapping_sub(result_at_0),
            0x200,
            "difference should equal the PC difference"
        );
    }
}
