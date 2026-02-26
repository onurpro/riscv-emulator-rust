use riscv_emulator_rust::RiscvCpu;

/// Encode a generic R-type ALU instruction.
///
/// funct7: high 7 bits of funct (e.g., 0b0000000 for ADD, 0b0100000 for SUB)
/// rs2: second source register
/// rs1: first source register
/// funct3: middle 3 bits of funct (e.g., 0b000 for ADD/SUB)
/// rd: destination register
/// opcode: usually 0x33 for OP
fn encode_rtype(funct7: u8, rs2: u8, rs1: u8, funct3: u8, rd: u8, opcode: u8) -> u32 {
    ((funct7 as u32) << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | ((rd as u32) << 7)
        | (opcode as u32)
}

mod add_sub {
    use super::*;

    #[test]
    fn test_add_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 5;

        // add x3, x1, x2  (x3 = 10 + 5 = 15)
        let inst = encode_rtype(0b0000000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 15);
        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_add_with_negative() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = -10i32 as u32;
        cpu.regs[2] = 3;

        // add x3, x1, x2  (x3 = -10 + 3 = -7)
        let inst = encode_rtype(0b0000000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3] as i32, -7);
    }

    #[test]
    fn test_add_overflow_wraps() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 1;

        // add x3, x1, x2  (x3 = 0xFFFF_FFFF + 1 = 0 (wrap))
        let inst = encode_rtype(0b0000000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_sub_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 5;

        // sub x3, x1, x2  (x3 = 10 - 5 = 5)
        let inst = encode_rtype(0b0100000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 5);
    }

    #[test]
    fn test_sub_negative_result() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;
        cpu.regs[2] = 10;

        // sub x3, x1, x2  (x3 = 5 - 10 = -5)
        let inst = encode_rtype(0b0100000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3] as i32, -5);
    }

    #[test]
    fn test_sub_underflow_wraps() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0;
        cpu.regs[2] = 1;

        // sub x3, x1, x2  (0 - 1 wraps to 0xFFFF_FFFF)
        let inst = encode_rtype(0b0100000, 2, 1, 0b000, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0xFFFF_FFFF);
    }

    #[test]
    fn test_add_does_not_write_x0() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 123;
        cpu.regs[2] = 456;

        // add x0, x1, x2  (must not modify x0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b000, 0, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_sub_does_not_write_x0() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 123;
        cpu.regs[2] = 456;

        // sub x0, x1, x2  (must not modify x0)
        let inst = encode_rtype(0b0100000, 2, 1, 0b000, 0, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_add_same_register_rs1_rs2_rd() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;

        // add x1, x1, x1  (x1 = 5 + 5 = 10, all three regs are the same)
        let inst = encode_rtype(0b0000000, 1, 1, 0b000, 1, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[1], 10);
    }

    #[test]
    fn test_add_result_into_source_register() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 3;
        cpu.regs[2] = 7;

        // add x1, x1, x2  (rd == rs1, x1 = 3 + 7 = 10)
        let inst = encode_rtype(0b0000000, 2, 1, 0b000, 1, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[1], 10);
        assert_eq!(cpu.regs[2], 7, "rs2 must be unchanged");
    }
}

mod sll {
    use super::*;

    #[test]
    fn test_sll_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 1;
        cpu.regs[2] = 3; // shift amount in low 5 bits

        // sll x3, x1, x2  (x3 = 1 << 3 = 8)
        let inst = encode_rtype(0b0000000, 2, 1, 0b001, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 8);
    }

    #[test]
    fn test_sll_uses_low5_bits() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 1;
        cpu.regs[2] = 0b1_00000; // bit 5 set, low 5 bits = 0

        // sll x3, x1, x2  (x3 = 1 << (0) = 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b001, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_sll_max_shift_31() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 1;
        cpu.regs[2] = 31;

        // sll x3, x1, x2  (x3 = 1 << 31)
        let inst = encode_rtype(0b0000000, 2, 1, 0b001, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1u32 << 31);
    }

    #[test]
    fn test_sll_with_zero() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0;
        cpu.regs[2] = 5;

        // sll x3, x1, x2  (0 << anything = 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b001, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }
}

mod slt_sltu {
    use super::*;

    #[test]
    fn test_slt_less_than_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 5;
        cpu.regs[2] = 10;

        // slt x3, x1, x2  (5 < 10 => 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b010, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_slt_greater_equal_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 10;
        cpu.regs[2] = 10;

        // slt x3, x1, x2  (10 < 10 => 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b010, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_slt_signed_negative_vs_positive() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = -1i32 as u32; // 0xFFFF_FFFF
        cpu.regs[2] = 1;

        // slt x3, x1, x2  (-1 < 1 => 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b010, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_sltu_unsigned() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 1;
        cpu.regs[2] = 0xFFFF_FFFF;

        // sltu x3, x1, x2  (1 < 0xFFFF_FFFF unsigned => 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b011, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_sltu_unsigned_zero_vs_nonzero() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0;
        cpu.regs[2] = 123;

        // sltu x3, x1, x2  (0 < 123 => 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b011, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_sltu_equal_values() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0xFFFF_FFFF;

        // sltu x3, x1, x2  (equal => 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b011, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_sltu_greater_than() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 1;

        // sltu x3, x1, x2  (0xFFFF_FFFF < 1? false => 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b011, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_slt_equal_values() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 42;
        cpu.regs[2] = 42;

        // slt x3, x1, x2  (42 < 42? false => 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b010, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }
}

mod xor_or_and {
    use super::*;

    #[test]
    fn test_xor_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1010;
        cpu.regs[2] = 0b0110;

        // xor x3, x1, x2  (1010 ^ 0110 = 1100)
        let inst = encode_rtype(0b0000000, 2, 1, 0b100, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0b1100);
    }

    #[test]
    fn test_xor_with_self() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xDEAD_BEEF;
        cpu.regs[2] = 0xDEAD_BEEF;

        // xor x3, x1, x2  (x ^ x = 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b100, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_or_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1010;
        cpu.regs[2] = 0b0110;

        // or x3, x1, x2  (1010 | 0110 = 1110)
        let inst = encode_rtype(0b0000000, 2, 1, 0b110, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0b1110);
    }

    #[test]
    fn test_or_with_zero() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0;
        cpu.regs[2] = 0x1234_5678;

        // or x3, x1, x2  (0 | x2 = x2)
        let inst = encode_rtype(0b0000000, 2, 1, 0b110, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0x1234_5678);
    }

    #[test]
    fn test_and_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1010;
        cpu.regs[2] = 0b0110;

        // and x3, x1, x2  (1010 & 0110 = 0010)
        let inst = encode_rtype(0b0000000, 2, 1, 0b111, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0b0010);
    }

    #[test]
    fn test_and_with_zero() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0;

        // and x3, x1, x2  (x & 0 = 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b111, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0);
    }

    #[test]
    fn test_xor_all_ones() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0x0000_0000;

        // xor x3, x1, x2  (0xFFFF_FFFF ^ 0 = 0xFFFF_FFFF)
        let inst = encode_rtype(0b0000000, 2, 1, 0b100, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0xFFFF_FFFF);
    }

    #[test]
    fn test_or_all_ones_absorbs() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0x1234_5678;

        // or x3, x1, x2  (all-ones | anything = all-ones)
        let inst = encode_rtype(0b0000000, 2, 1, 0b110, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0xFFFF_FFFF);
    }

    #[test]
    fn test_and_all_ones_identity() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0xFFFF_FFFF;

        // and x3, x1, x2  (all-ones & all-ones = all-ones)
        let inst = encode_rtype(0b0000000, 2, 1, 0b111, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0xFFFF_FFFF);
    }
}

mod srl_sra {
    use super::*;

    #[test]
    fn test_srl_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0b1000;
        cpu.regs[2] = 3;

        // srl x3, x1, x2  (1000 >> 3 = 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_srl_zero_fill() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x8000_0000; // MSB set
        cpu.regs[2] = 31;

        // srl x3, x1, x2  (logical shift, result = 1)
        let inst = encode_rtype(0b0000000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 1);
    }

    #[test]
    fn test_srl_uses_low5_bits() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xF000_0000;
        cpu.regs[2] = 0b1_00000; // low 5 bits = 0

        // srl x3, x1, x2  (shift by 0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0xF000_0000);
    }

    #[test]
    fn test_sra_basic() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = (-8i32) as u32; // 0xFFFF_FFF8
        cpu.regs[2] = 1;

        // sra x3, x1, x2  (arithmetic shift, preserve sign)
        let inst = encode_rtype(0b0100000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        // -8 >> 1 = -4
        assert_eq!(cpu.regs[3] as i32, -4);
    }

    #[test]
    fn test_sra_shift_large() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = (-1i32) as u32; // all ones
        cpu.regs[2] = 31;

        // sra x3, x1, x2  (-1 >> 31 = -1 for arithmetic shift)
        let inst = encode_rtype(0b0100000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3] as i32, -1);
    }

    #[test]
    fn test_sra_uses_low5_bits() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = (-16i32) as u32;
        cpu.regs[2] = 0b1_00000; // low 5 bits = 0

        // sra x3, x1, x2  (shift by 0)
        let inst = encode_rtype(0b0100000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], cpu.regs[1]);
    }

    #[test]
    fn test_sra_positive_value() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0x0000_0040; // 64
        cpu.regs[2] = 2;

        // sra x3, x1, x2  (64 >> 2 = 16, positive: behaves same as SRL)
        let inst = encode_rtype(0b0100000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 16);
        // High bit must not be set for a positive input
        assert_eq!(cpu.regs[3] & 0x8000_0000, 0);
    }

    #[test]
    fn test_srl_all_ones_zero_fill() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 4;

        // srl x3, x1, x2  (0xFFFF_FFFF >> 4 = 0x0FFF_FFFF, zero-fill)
        let inst = encode_rtype(0b0000000, 2, 1, 0b101, 3, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[3], 0x0FFF_FFFF);
    }
}

mod x0_behavior {
    use super::*;

    #[test]
    fn test_rtype_reading_x0() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[0] = 0;
        cpu.regs[1] = 123;

        // add x2, x0, x1  (x2 = 0 + 123)
        let inst = encode_rtype(0b0000000, 1, 0, 0b000, 2, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[2], 123);
        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_rtype_writing_to_x0_from_nonzero_regs() {
        let mut cpu = RiscvCpu::new();
        cpu.regs[1] = 0xFFFF_FFFF;
        cpu.regs[2] = 0xFFFF_FFFF;

        // and x0, x1, x2  (should not change x0)
        let inst = encode_rtype(0b0000000, 2, 1, 0b111, 0, 0x33);
        cpu.handle_rtype(inst);

        assert_eq!(cpu.regs[0], 0);
    }
}
