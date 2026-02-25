use riscv_emulator_rust::RiscvCpu;

/// Encode a generic I-type ALU instruction (ADDI/SLTI/SLTIU/XORI/ORI/ANDI).
///
/// imm: 12-bit signed immediate
/// rs1: source register
/// funct3: instruction funct3
/// rd: destination register
/// opcode: usually 0x13 for ALU-immediate
fn encode_itype_imm(imm: i32, rs1: u8, funct3: u8, rd: u8, opcode: u8) -> u32 {
    let imm12 = (imm as i32) & 0xFFF; // take low 12 bits (two's complement)
    ((imm12 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | ((rd as u32) << 7)
        | (opcode as u32)
}

/// Encode shift-immediate instructions (SLLI/SRLI/SRAI).
///
/// shamt: shift amount (0–31 for RV32)
/// rs1: source register
/// rd: destination register
/// funct3: 0b001 for SLLI, 0b101 for SRLI/SRAI
/// arith: true for SRAI (funct7=0b0100000), false for SLLI/SRLI (funct7=0)
fn encode_shift_itype(shamt: u8, rs1: u8, rd: u8, funct3: u8, arith: bool) -> u32 {
    let funct7 = if arith { 0b0100000u32 } else { 0b0000000u32 };
    let imm12 = (funct7 << 5) | (shamt as u32);
    (imm12 << 20) | ((rs1 as u32) << 15) | ((funct3 as u32) << 12) | ((rd as u32) << 7) | 0x13u32 // opcode for OP-IMM
}

mod addi {
    use super::*;

    #[test]
    fn test_addi_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;

        // addi x2, x1, 5  (x2 = 10 + 5 = 15)
        let instruction = encode_itype_imm(5, 1, 0b000, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 15);
        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_addi_negative() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;

        // addi x2, x1, -1  (x2 = 10 + (-1) = 9)
        let instruction = encode_itype_imm(-1, 1, 0b000, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 9);
    }

    #[test]
    fn test_addi_zero_imm() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[3] = 0x1234_5678;

        // addi x4, x3, 0  (MV pseudo-op)
        let instruction = encode_itype_imm(0, 3, 0b000, 4, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[4], 0x1234_5678);
    }

    #[test]
    fn test_addi_does_not_write_x0() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 123;

        // addi x0, x1, 5  (must be ignored; x0 always zero)
        let instruction = encode_itype_imm(5, 1, 0b000, 0, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[0], 0);
    }
}

mod slti {
    use super::*;

    #[test]
    fn test_slti_less_than() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;

        // slti x2, x1, 10  (5 < 10 => x2 = 1)
        let instruction = encode_itype_imm(10, 1, 0b010, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 1);
    }

    #[test]
    fn test_slti_not_less_than() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 20;

        // slti x2, x1, 10  (20 < 10? false => x2 = 0)
        let instruction = encode_itype_imm(10, 1, 0b010, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0);
    }

    #[test]
    fn test_slti_signed_negative_vs_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = -5i32 as u32;

        // slti x2, x1, 0  (-5 < 0 => x2 = 1, signed compare)
        let instruction = encode_itype_imm(0, 1, 0b010, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 1);
    }    
}

mod sltiu {
    use super::*;

    #[test]
    fn test_sltiu_unsigned_less_than() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;

        // sltiu x2, x1, 10  (5 < 10 => x2 = 1, unsigned)
        let instruction = encode_itype_imm(10, 1, 0b011, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 1);
    }

    #[test]
    fn test_sltiu_negative_is_large_unsigned() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF; // -1 as signed, max as unsigned

        // sltiu x2, x1, 0  (0xFFFF_FFFF < 0 ? false => 0, unsigned)
        let instruction = encode_itype_imm(0, 1, 0b011, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0);
}

#[test]
fn test_xori_basic() {
    let mut cpu = RiscvCpu::new();
    cpu.regs[1] = 0b1010;

    // xori x2, x1, 0b0110  => 0b1010 ^ 0b0110 = 0b1100 (12)
    let instruction = encode_itype_imm(0b0110, 1, 0b100, 2, 0x13);

    cpu.handle_itype(instruction);

    assert_eq!(cpu.regs[2], 0b1100);
}

#[test]
fn test_xori_not_pseudo() {
    let mut cpu = RiscvCpu::new();
    cpu.regs[1] = 0x1234_5678;

    // xori x2, x1, -1  => bitwise NOT
    let instruction = encode_itype_imm(-1, 1, 0b100, 2, 0x13);

    cpu.handle_itype(instruction);

    assert_eq!(cpu.regs[2], !0x1234_5678);
}
}

mod ori {
    use super::*;

    #[test]
    fn test_ori_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1001;

        // ori x2, x1, 0b0110 => 0b1001 | 0b0110 = 0b1111
        let instruction = encode_itype_imm(0b0110, 1, 0b110, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0b1111);
    }

    #[test]
    fn test_ori_with_zero() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xDEAD_BEEF;

        // ori x2, x1, 0 => x2 = x1
        let instruction = encode_itype_imm(0, 1, 0b110, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0xDEAD_BEEF);
    }
}

mod andi {
    use super::*;

    #[test]
    fn test_andi_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1101;

        // andi x2, x1, 0b0110 => 0b1101 & 0b0110 = 0b0100
        let instruction = encode_itype_imm(0b0110, 1, 0b111, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0b0100);
    }

    #[test]
    fn test_andi_masking() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;

        // andi x2, x1, 0x0FF  => low 8 bits set, others cleared
        let instruction = encode_itype_imm(0x0FF, 1, 0b111, 2, 0x13);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0xFF);
    }
}

mod slli {
    use super::*;

    #[test]
    fn test_slli_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1;

        // slli x2, x1, 3  => 1 << 3 = 8
        let instruction = encode_shift_itype(3, 1, 2, 0b001, false);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 8);
    }

    #[test]
    fn test_slli_zero_shamt() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x1234_5678;

        // slli x2, x1, 0  => no change
        let instruction = encode_shift_itype(0, 1, 2, 0b001, false);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0x1234_5678);
    }
}

mod srli {
    use super::*;

    #[test]
    fn test_srli_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1000;

        // srli x2, x1, 3  => 0b1000 >> 3 = 0b1
        let instruction = encode_shift_itype(3, 1, 2, 0b101, false);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0b1);
    }

    #[test]
    fn test_srli_zero_fill_on_negative() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x8000_0000; // MSB set

        // srli x2, x1, 1 => logical shift, new MSB must be 0
        let instruction = encode_shift_itype(1, 1, 2, 0b101, false);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0x4000_0000);
    }
}

mod srai {
    use super::*;

    #[test]
    fn test_srai_negative() {
        let mut cpu = RiscvCpu::new();
        // x1 = -10 (0xFFFFFFF6)
        cpu.regs[1] = 0xFFFF_FFF6;

        // srai x2, x1, 1  => -10 >> 1 = -5 (0xFFFFFFFB)
        let instruction = encode_shift_itype(1, 1, 2, 0b101, true);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 0xFFFF_FFFB);
    }

    #[test]
    fn test_srai_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x0000_0008;

        // srai x2, x1, 1  => 8 >> 1 = 4
        let instruction = encode_shift_itype(1, 1, 2, 0b101, true);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2], 4);
    }

    #[test]
    fn test_srai_sign_bit_preserved() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x8000_0000; // -2147483648

        // srai x2, x1, 4  => arithmetic shift keeps sign bit set
        let instruction = encode_shift_itype(4, 1, 2, 0b101, true);

        cpu.handle_itype(instruction);

        assert_eq!(cpu.regs[2] & 0x8000_0000, 0x8000_0000);
    }
}
