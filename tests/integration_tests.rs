use riscv_emulator_rust::RiscvCpu;

// ── Instruction encoders ──────────────────────────────────────────────────────

fn encode_rtype(funct7: u8, rs2: u8, rs1: u8, funct3: u8, rd: u8) -> u32 {
    ((funct7 as u32) << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | ((rd as u32) << 7)
        | 0x33
}

fn encode_itype(imm: i32, rs1: u8, funct3: u8, rd: u8) -> u32 {
    let imm12 = (imm as i32) & 0xFFF;
    ((imm12 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | ((rd as u32) << 7)
        | 0x13
}

fn encode_load(imm: i32, rs1: u8, funct3: u8, rd: u8) -> u32 {
    let imm12 = (imm as i32) & 0xFFF;
    ((imm12 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | ((rd as u32) << 7)
        | 0x03
}

fn encode_btype(imm: i32, rs1: u8, rs2: u8, funct3: u8) -> u32 {
    let imm = imm as u32;
    let b12 = (imm >> 12) & 0x1;
    let b11 = (imm >> 11) & 0x1;
    let b10_5 = (imm >> 5) & 0x3F;
    let b4_1 = (imm >> 1) & 0xF;

    (b12 << 31)
        | (b10_5 << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((funct3 as u32) << 12)
        | (b4_1 << 8)
        | (b11 << 7)
        | 0x63
}

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

fn encode_jal(imm: i32, rd: u8) -> u32 {
    let imm = imm as u32;
    let i20 = (imm >> 20) & 0x1;
    let i19_12 = (imm >> 12) & 0xFF;
    let i11 = (imm >> 11) & 0x1;
    let i10_1 = (imm >> 1) & 0x3FF;

    (i20 << 31) | (i10_1 << 21) | (i11 << 20) | (i19_12 << 12) | ((rd as u32) << 7) | 0x6F
}

fn encode_jalr(imm: i32, rs1: u8, rd: u8) -> u32 {
    let imm12 = (imm & 0xFFF) as u32;
    (imm12 << 20) | ((rs1 as u32) << 15) | (0b000u32 << 12) | ((rd as u32) << 7) | 0x67
}

fn encode_lui(imm: u32, rd: u8) -> u32 {
    ((imm & 0xFFFFF) << 12) | ((rd as u32) << 7) | 0x37
}

fn encode_auipc(imm: u32, rd: u8) -> u32 {
    ((imm & 0xFFFFF) << 12) | ((rd as u32) << 7) | 0x17
}

// ── Helper: write a program into the CPU's bus starting at address 0 ──────────

fn load_program(cpu: &mut RiscvCpu, instructions: &[u32]) {
    for (i, &inst) in instructions.iter().enumerate() {
        let addr = i * 4;
        let bytes = inst.to_le_bytes();
        cpu.bus[addr..addr + 4].copy_from_slice(&bytes);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A chain of I-Type instructions where the output register of one feeds
/// the input of the next — this can only work correctly if `step()` properly
/// updates the register file between instructions.
///
/// ADDI x1, x0, 10   ; x1 = 10
/// ADDI x2, x1, 5    ; x2 = 15
/// ADDI x3, x2, -3   ; x3 = 12
#[test]
fn test_itype_chain() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(10, 0, 0b000, 1), // addi x1, x0, 10
        encode_itype(5, 1, 0b000, 2),  // addi x2, x1, 5
        encode_itype(-3, 2, 0b000, 3), // addi x3, x2, -3
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // addi x1
    cpu.step().unwrap(); // addi x2
    cpu.step().unwrap(); // addi x3

    assert_eq!(cpu.regs[1], 10, "x1 should be 10");
    assert_eq!(cpu.regs[2], 15, "x2 should be 15");
    assert_eq!(cpu.regs[3], 12, "x3 should be 12");
}

/// An I-Type instruction sets up a register, then an R-Type instruction
/// consumes it — tests cross-type data dependency.
///
/// ADDI x1, x0, 20   ; x1 = 20
/// ADDI x2, x0, 7    ; x2 = 7
/// ADD  x3, x1, x2   ; x3 = 27
/// SUB  x4, x3, x2   ; x4 = 20
#[test]
fn test_itype_then_rtype() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(20, 0, 0b000, 1),           // addi x1, x0, 20
        encode_itype(7, 0, 0b000, 2),            // addi x2, x0, 7
        encode_rtype(0b0000000, 2, 1, 0b000, 3), // add  x3, x1, x2
        encode_rtype(0b0100000, 2, 3, 0b000, 4), // sub  x4, x3, x2
    ];
    load_program(&mut cpu, &program);

    for _ in 0..4 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[3], 27, "x3 should be 27 (20 + 7)");
    assert_eq!(cpu.regs[4], 20, "x4 should be 20 (27 - 7)");
}

/// R-Type instructions using the results of earlier R-Type instructions.
///
/// ADDI x1, x0, 0b1010   ; x1 = 10
/// ADDI x2, x0, 0b1100   ; x2 = 12
/// XOR  x3, x1, x2        ; x3 = 0b0110 (6)
/// OR   x4, x1, x2        ; x4 = 0b1110 (14)
/// AND  x5, x3, x4        ; x5 = 0b0110 & 0b1110 = 0b0110 (6)
#[test]
fn test_rtype_chain() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(0b1010, 0, 0b000, 1),       // addi x1, x0, 10
        encode_itype(0b1100, 0, 0b000, 2),       // addi x2, x0, 12
        encode_rtype(0b0000000, 2, 1, 0b100, 3), // xor  x3, x1, x2
        encode_rtype(0b0000000, 2, 1, 0b110, 4), // or   x4, x1, x2
        encode_rtype(0b0000000, 4, 3, 0b111, 5), // and  x5, x3, x4
    ];
    load_program(&mut cpu, &program);

    for _ in 0..5 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[3], 0b0110, "XOR result should be 0b0110");
    assert_eq!(cpu.regs[4], 0b1110, "OR result should be 0b1110");
    assert_eq!(
        cpu.regs[5], 0b0110,
        "AND of XOR/OR results should be 0b0110"
    );
}

/// A B-Type branch that is taken jumps over the next instruction.
///
/// Program layout (address 0):
///   0x00: ADDI x1, x0, 5     ; x1 = 5
///   0x04: ADDI x2, x0, 5     ; x2 = 5
///   0x08: BEQ  x1, x2, +8    ; branch to 0x10 (skip the next instruction)
///   0x0C: ADDI x3, x0, 99    ; should be SKIPPED
///   0x10: ADDI x4, x0, 42    ; should execute
#[test]
fn test_branch_taken_skips_instruction() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(5, 0, 0b000, 1),  // 0x00: addi x1, x0, 5
        encode_itype(5, 0, 0b000, 2),  // 0x04: addi x2, x0, 5
        encode_btype(8, 1, 2, 0b000),  // 0x08: beq  x1, x2, +8  → 0x10
        encode_itype(99, 0, 0b000, 3), // 0x0C: addi x3, x0, 99  (skipped)
        encode_itype(42, 0, 0b000, 4), // 0x10: addi x4, x0, 42
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // addi x1
    cpu.step().unwrap(); // addi x2
    cpu.step().unwrap(); // beq  — branch taken, pc → 0x10
    cpu.step().unwrap(); // addi x4 (at 0x10)

    assert_eq!(cpu.regs[3], 0, "x3 should be 0 (skipped instruction)");
    assert_eq!(cpu.regs[4], 42, "x4 should be 42 (executed after branch)");
    assert_eq!(cpu.pc, 0x14, "PC should be 0x14 after last step");
}

/// A B-Type branch that is NOT taken falls through normally.
///
///   0x00: ADDI x1, x0, 5
///   0x04: ADDI x2, x0, 9
///   0x08: BEQ  x1, x2, +8   ; not taken (5 ≠ 9), fall through
///   0x0C: ADDI x3, x0, 77   ; should execute
#[test]
fn test_branch_not_taken_falls_through() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(5, 0, 0b000, 1),  // 0x00: addi x1, x0, 5
        encode_itype(9, 0, 0b000, 2),  // 0x04: addi x2, x0, 9
        encode_btype(8, 1, 2, 0b000),  // 0x08: beq  x1, x2, +8 (not taken)
        encode_itype(77, 0, 0b000, 3), // 0x0C: addi x3, x0, 77
    ];
    load_program(&mut cpu, &program);

    for _ in 0..4 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[3], 77, "x3 should be 77 (fall-through executed)");
    assert_eq!(cpu.pc, 0x10, "PC should point past the last instruction");
}

/// A countdown loop using BNE:
///
///   x1 = 5 (counter)
///   x2 = 0 (zero register value via ADDI x2, x0, 0)
///   loop:
///     ADDI x1, x1, -1        ; decrement
///     ADDI x3, x3, 1         ; accumulate iteration count
///     BNE  x1, x2, -8        ; loop back if x1 ≠ 0
///
/// After 5 iterations: x1 = 0, x3 = 5.
#[test]
fn test_countdown_loop() {
    let mut cpu = RiscvCpu::new();

    // Initialisation (before loop)
    //   0x00: addi x1, x0, 5
    //   0x04: addi x2, x0, 0   (x2 stays 0)
    // Loop body starts at 0x08
    //   0x08: addi x1, x1, -1
    //   0x0C: addi x3, x3, 1
    //   0x10: bne  x1, x2, -8  ; branch back to 0x08 while x1 ≠ 0
    let program = [
        encode_itype(5, 0, 0b000, 1),  // 0x00: addi x1, x0, 5
        encode_itype(0, 0, 0b000, 2),  // 0x04: addi x2, x0, 0
        encode_itype(-1, 1, 0b000, 1), // 0x08: addi x1, x1, -1
        encode_itype(1, 3, 0b000, 3),  // 0x0C: addi x3, x3, 1
        encode_btype(-8, 1, 2, 0b001), // 0x10: bne  x1, x2, -8
    ];
    load_program(&mut cpu, &program);

    // 2 setup instructions
    cpu.step().unwrap();
    cpu.step().unwrap();

    // 5 loop iterations × 3 instructions each = 15 steps
    for _ in 0..15 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[1], 0, "counter x1 should reach 0");
    assert_eq!(
        cpu.regs[3], 5,
        "accumulator x3 should be 5 after 5 iterations"
    );
}

#[test]
fn test_blt_bge_loop() {
    // Tests a loop that goes from x1 = 0 to x1 = 5.
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(0, 0, 0b000, 1), // 0x00: addi x1, x0, 0
        encode_itype(5, 0, 0b000, 2), // 0x04: addi x2, x0, 5
        // Loop start (0x08)
        encode_btype(12, 1, 2, 0b101), // 0x08: bge x1, x2, +12 (to 0x14)
        encode_itype(1, 1, 0b000, 1),  // 0x0C: addi x1, x1, 1
        encode_btype(-8, 1, 2, 0b100), // 0x10: blt x1, x2, -8 (to 0x08)
        // End (0x14)
        encode_itype(100, 0, 0b000, 3), // 0x14: addi x3, x0, 100
    ];

    load_program(&mut cpu, &program);

    // Initial setup: 2 instructions
    cpu.step().unwrap(); // x1 = 0
    cpu.step().unwrap(); // x2 = 5

    // 5 iterations x 3 steps each
    for _ in 0..(5 * 3) {
        cpu.step().unwrap();
    }

    // Final step at 0x14
    cpu.step().unwrap();

    assert_eq!(cpu.regs[1], 5, "x1 should be 5");
    assert_eq!(
        cpu.regs[3], 100,
        "x3 should be 100, indicating loop finished and reached end"
    );
}

#[test]
fn test_bltu_bgeu_unsigned_comparisons() {
    let mut cpu = RiscvCpu::new();

    // unsigned comparison with negative numbers (which are large positive in unsigned)
    let program = [
        encode_itype(-1, 0, 0b000, 1), // 0x00: addi x1, x0, -1
        encode_itype(1, 0, 0b000, 2),  // 0x04: addi x2, x0, 1
        encode_btype(8, 1, 2, 0b111),  // 0x08: bgeu x1, x2, +8 (to 0x10)
        encode_itype(99, 0, 0b000, 3), // 0x0C: addi x3, x0, 99 (skipped)
        encode_btype(8, 2, 1, 0b110),  // 0x10: bltu x2, x1, +8 (to 0x18)
        encode_itype(99, 0, 0b000, 4), // 0x14: addi x4, x0, 99 (skipped)
        encode_itype(42, 0, 0b000, 5), // 0x18: addi x5, x0, 42
    ];

    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // addi x1
    cpu.step().unwrap(); // addi x2
    cpu.step().unwrap(); // bgeu
    cpu.step().unwrap(); // bltu
    cpu.step().unwrap(); // addi x5

    assert_eq!(cpu.regs[3], 0, "x3 should be 0 (skipped)");
    assert_eq!(cpu.regs[4], 0, "x4 should be 0 (skipped)");
    assert_eq!(cpu.regs[5], 42, "x5 should be 42 (executed)");
}

#[test]
fn test_load_instructions_integration() {
    let mut cpu = RiscvCpu::new();

    // Set up some data memory
    cpu.bus[0x200] = 0x11;
    cpu.bus[0x201] = 0x22;
    cpu.bus[0x202] = 0x33;
    cpu.bus[0x203] = 0x44; // Word at 0x200 is 0x44332211

    cpu.bus[0x204] = 0xFF;
    cpu.bus[0x205] = 0xEE;
    cpu.bus[0x206] = 0xDD;
    cpu.bus[0x207] = 0xCC; // Word at 0x204 is 0xCCDDEEFF

    let program = [
        encode_itype(0x200, 0, 0b000, 1),        // 0x00: addi x1, x0, 0x200
        encode_load(0, 1, 0b010, 2),             // 0x04: lw x2, 0(x1)      (x2 = 0x44332211)
        encode_load(4, 1, 0b001, 3), // 0x08: lh x3, 4(x1)      (x3 = sign_extend(0xEEFF) = 0xFFFFEEFF)
        encode_load(5, 1, 0b100, 4), // 0x0C: lbu x4, 5(x1)     (x4 = lbu from 0x205 = 0xEE)
        encode_rtype(0b0000000, 4, 3, 0b000, 5), // 0x10: add x5, x3, x4 (x5 = 0xFFFFEEFF + 0xEE = 0xFFFFEFED)
    ];

    load_program(&mut cpu, &program);

    for _ in 0..5 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[2], 0x4433_2211, "x2 should be the loaded word");
    assert_eq!(
        cpu.regs[3], 0xFFFF_EEFF,
        "x3 should be sign-extended halfword"
    );
    assert_eq!(
        cpu.regs[4], 0xEE,
        "x4 should be unsigned byte zero-extended"
    );
    assert_eq!(cpu.regs[5], 0xFFFF_EFED, "x5 should be sum of x3 and x4");
}

#[test]
fn test_store_instructions_integration() {
    let mut cpu = RiscvCpu::new();

    // Setup:
    // x1 = base address 0x200
    // x2 = 0x08090A0B (word value)
    // x3 = 0x0C0D (halfword value)
    // x4 = 0x0E (byte value)

    cpu.regs[1] = 0x200;
    cpu.regs[2] = 0x08090A0B;
    cpu.regs[3] = 0x0C0D;
    cpu.regs[4] = 0x0E;

    let program = [
        encode_stype(0, 2, 1, 0b010), // 0x00: sw x2, 0(x1)
        encode_stype(4, 3, 1, 0b001), // 0x04: sh x3, 4(x1)
        encode_stype(6, 4, 1, 0b000), // 0x08: sb x4, 6(x1)
        // Now load them back into new registers to verify memory AND load interactions work
        encode_load(0, 1, 0b010, 5), // 0x0C: lw x5, 0(x1)
        encode_load(4, 1, 0b001, 6), // 0x10: lh x6, 4(x1)
        encode_load(6, 1, 0b100, 7), // 0x14: lbu x7, 6(x1)
    ];

    load_program(&mut cpu, &program);

    for _ in 0..6 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[5], 0x08090A0B, "x5 should hold loaded word");
    // Sign extension from loaded halfword (0x0C0D) goes to 0x00000C0D since MSB of 0x0C is 0
    assert_eq!(cpu.regs[6], 0x0C0D, "x6 should hold loaded halfword");
    assert_eq!(cpu.regs[7], 0x0E, "x7 should hold loaded unsigned byte");

    // We can also verify bus manually
    assert_eq!(cpu.bus[0x200], 0x0B);
    assert_eq!(cpu.bus[0x201], 0x0A);
    assert_eq!(cpu.bus[0x202], 0x09);
    assert_eq!(cpu.bus[0x203], 0x08);

    assert_eq!(cpu.bus[0x204], 0x0D);
    assert_eq!(cpu.bus[0x205], 0x0C);

    assert_eq!(cpu.bus[0x206], 0x0E);
}

/// Fibonacci sequence via unrolled ADD chain:
///
///   x1=0, x2=1
///   x3 = x1+x2 = 1   (F2)
///   x4 = x2+x3 = 2   (F3)
///   x5 = x3+x4 = 3   (F4)
///   x6 = x4+x5 = 5   (F5)
///   x7 = x5+x6 = 8   (F6)
///   x8 = x6+x7 = 13  (F7)
///
/// Verifies that ADD correctly chains through the register file.
#[test]
fn test_fibonacci_sequence() {
    let mut cpu = RiscvCpu::new();

    let program = [
        encode_itype(0, 0, 0b000, 1),            // 0x00: addi x1, x0, 0  (F0=0)
        encode_itype(1, 0, 0b000, 2),            // 0x04: addi x2, x0, 1  (F1=1)
        encode_rtype(0b0000000, 2, 1, 0b000, 3), // 0x08: add  x3, x1, x2 (F2=1)
        encode_rtype(0b0000000, 3, 2, 0b000, 4), // 0x0C: add  x4, x2, x3 (F3=2)
        encode_rtype(0b0000000, 4, 3, 0b000, 5), // 0x10: add  x5, x3, x4 (F4=3)
        encode_rtype(0b0000000, 5, 4, 0b000, 6), // 0x14: add  x6, x4, x5 (F5=5)
        encode_rtype(0b0000000, 6, 5, 0b000, 7), // 0x18: add  x7, x5, x6 (F6=8)
        encode_rtype(0b0000000, 7, 6, 0b000, 8), // 0x1C: add  x8, x6, x7 (F7=13)
    ];
    load_program(&mut cpu, &program);

    for _ in 0..8 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[1], 0, "x1=F(0)=0");
    assert_eq!(cpu.regs[2], 1, "x2=F(1)=1");
    assert_eq!(cpu.regs[3], 1, "x3=F(2)=1");
    assert_eq!(cpu.regs[4], 2, "x4=F(3)=2");
    assert_eq!(cpu.regs[5], 3, "x5=F(4)=3");
    assert_eq!(cpu.regs[6], 5, "x6=F(5)=5");
    assert_eq!(cpu.regs[7], 8, "x7=F(6)=8");
    assert_eq!(cpu.regs[8], 13, "x8=F(7)=13");
}

/// Data-dependent branch: compute a value, then branch if it equals an expected value.
///
///   x1 = 3
///   x2 = 4
///   x3 = x1 + x2        ; 7
///   x4 = x3 - 7         ; 0
///   if (x4 == x0) branch over addi x5, x0, 99 → addi x5, x0, 42
#[test]
fn test_data_dependent_branch() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(3, 0, 0b000, 1),            // 0x00: addi x1, x0, 3
        encode_itype(4, 0, 0b000, 2),            // 0x04: addi x2, x0, 4
        encode_rtype(0b0000000, 2, 1, 0b000, 3), // 0x08: add  x3, x1, x2  (x3=7)
        encode_itype(-7, 3, 0b000, 4),           // 0x0C: addi x4, x3, -7  (x4=0)
        encode_btype(8, 4, 0, 0b000),            // 0x10: beq  x4, x0, +8  → 0x18
        encode_itype(99, 0, 0b000, 5),           // 0x14: addi x5, x0, 99  (skipped)
        encode_itype(42, 0, 0b000, 5),           // 0x18: addi x5, x0, 42
    ];
    load_program(&mut cpu, &program);

    for _ in 0..6 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[3], 7, "x3 should be 7 (sum)");
    assert_eq!(cpu.regs[4], 0, "x4 should be 0 (sum - 7)");
    assert_eq!(cpu.regs[5], 42, "x5 should be 42 (branch was taken)");
}

/// Verify that writing to x0 via step() always keeps x0 = 0.
///
///   addi x0, x1, 5   ; should be silently discarded
#[test]
fn test_x0_write_protection_via_step() {
    let mut cpu = RiscvCpu::new();
    cpu.regs[1] = 100;

    let program = [
        encode_itype(5, 1, 0b000, 0), // 0x00: addi x0, x1, 5  (rd=0, ignored)
        encode_itype(7, 0, 0b000, 2), // 0x04: addi x2, x0, 7  (x2 must read 0)
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // addi x0 — ignored
    cpu.step().unwrap(); // addi x2, x0, 7  => x2 = 0 + 7 = 7

    assert_eq!(cpu.regs[0], 0, "x0 must always be 0");
    assert_eq!(cpu.regs[2], 7, "x2 = 0 + 7 = 7 (confirmed x0 reads as 0)");
}

/// Multi-step ALU chain with shift instructions.
///
///   x1 = 1
///   x2 = x1 << 4           ; 16
///   x3 = x2 | 0b111        ; 16 | 7 = 23
///   x4 = x3 >> 1           ; 23 >> 1 = 11 (unsigned)
///   x5 = x4 & 0xF          ; 11 & 15 = 11
#[test]
fn test_shift_and_bitwise_chain() {
    let mut cpu = RiscvCpu::new();

    fn encode_slli_step(shamt: u8, rs1: u8, rd: u8) -> u32 {
        let funct7 = 0b0000000u32;
        let imm12 = (funct7 << 5) | (shamt as u32);
        (imm12 << 20) | ((rs1 as u32) << 15) | (0b001u32 << 12) | ((rd as u32) << 7) | 0x13u32
    }

    fn encode_srli_step(shamt: u8, rs1: u8, rd: u8) -> u32 {
        let funct7 = 0b0000000u32;
        let imm12 = (funct7 << 5) | (shamt as u32);
        (imm12 << 20) | ((rs1 as u32) << 15) | (0b101u32 << 12) | ((rd as u32) << 7) | 0x13u32
    }

    let program = [
        encode_itype(1, 0, 0b000, 1),     // addi x1, x0, 1
        encode_slli_step(4, 1, 2),        // slli x2, x1, 4  => 16
        encode_itype(0b111, 2, 0b110, 3), // ori  x3, x2, 7  => 16 | 7 = 23
        encode_srli_step(1, 3, 4),        // srli x4, x3, 1  => 23 >> 1 = 11
        encode_itype(0xF, 4, 0b111, 5),   // andi x5, x4, 15 => 11 & 15 = 11
    ];
    load_program(&mut cpu, &program);

    for _ in 0..5 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[2], 16, "x2 = 1 << 4 = 16");
    assert_eq!(cpu.regs[3], 23, "x3 = 16 | 7 = 23");
    assert_eq!(cpu.regs[4], 11, "x4 = 23 >> 1 = 11");
    assert_eq!(cpu.regs[5], 11, "x5 = 11 & 15 = 11");
}

// ── JAL / JALR / LUI / AUIPC integration tests ───────────────────────────────

/// JAL: jump forward over one instruction and verify the link register.
///
///   0x00: addi x1, x0, 1    ; x1 = 1
///   0x04: jal  x5, +8       ; x5 = 0x08 (PC+4), PC -> 0x0C
///   0x08: addi x2, x0, 99   ; skipped
///   0x0C: addi x3, x0, 42   ; executes
#[test]
fn test_jal_forward_jump_and_link() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(1, 0, 0b000, 1),  // 0x00: addi x1, x0, 1
        encode_jal(8, 5),              // 0x04: jal  x5, +8 -> 0x0C; x5=0x08
        encode_itype(99, 0, 0b000, 2), // 0x08: addi x2, x0, 99  (skipped)
        encode_itype(42, 0, 0b000, 3), // 0x0C: addi x3, x0, 42
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // addi x1
    cpu.step().unwrap(); // jal  x5  (jumps to 0x0C)
    cpu.step().unwrap(); // addi x3  (at 0x0C)

    assert_eq!(cpu.regs[1], 1, "x1 should be 1 (ran before jump)");
    assert_eq!(cpu.regs[5], 0x08, "x5 = link register = 0x04 + 4");
    assert_eq!(cpu.regs[2], 0, "x2 should be 0 (skipped)");
    assert_eq!(cpu.regs[3], 42, "x3 should be 42 (ran after jump)");
}

/// Simulate a call/return idiom with JAL + JALR.
///
///   0x00: jal  x1, +8    ; call subroutine at 0x08; x1 = 0x04
///   0x04: addi x2, x0, 7 ; runs after return
///   0x08: addi x3, x0, 5 ; subroutine body
///   0x0C: jalr x0, x1, 0 ; return (jump to x1 = 0x04)
#[test]
fn test_jal_jalr_call_return() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_jal(8, 1),             // 0x00: jal  x1, +8 -> 0x08; x1=0x04
        encode_itype(7, 0, 0b000, 2), // 0x04: addi x2, x0, 7
        encode_itype(5, 0, 0b000, 3), // 0x08: addi x3, x0, 5 (subroutine)
        encode_jalr(0, 1, 0),         // 0x0C: jalr x0, x1, 0 (return)
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // jal  x1  (call, jumps to 0x08)
    cpu.step().unwrap(); // addi x3  (subroutine body)
    cpu.step().unwrap(); // jalr x0  (return to 0x04)
    cpu.step().unwrap(); // addi x2  (post-return code)

    assert_eq!(cpu.regs[1], 0x04, "x1 = return address 0x04");
    assert_eq!(cpu.regs[3], 5, "x3 = 5 (subroutine ran)");
    assert_eq!(cpu.regs[2], 7, "x2 = 7 (post-return code ran)");
}

/// LUI + ADDI: standard RISC-V idiom for loading a 32-bit constant.
///
///   lui  x1, 0x12345   ; x1 = 0x12345000
///   addi x1, x1, 0x678 ; x1 = 0x12345678
#[test]
fn test_lui_addi_load_32bit_constant() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_lui(0x12345, 1),           // lui  x1, 0x12345 -> 0x12345000
        encode_itype(0x678, 1, 0b000, 1), // addi x1, x1, 0x678
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap();
    cpu.step().unwrap();

    assert_eq!(cpu.regs[1], 0x1234_5678, "x1 should hold 0x12345678");
}

/// LUI + sign-compensated ADDI: compiler idiom when low 12 bits have MSB set.
///
/// Target: 0xDEADBEEF
///   low 12 bits: 0xEEF -> bit-11 is set -> signed = 0xEEF - 0x1000 = -0x111
///   upper after compensation = (0xDEADBEEF + 0x800) >> 12 = 0xDEADC
///   lui  x1, 0xDEADC   -> x1 = 0xDEADC000
///   addi x1, x1, -0x111 -> x1 = 0xDEADBEEF
#[test]
fn test_lui_addi_sign_compensation() {
    let mut cpu = RiscvCpu::new();
    // 0xDEAD_BEEF: low 12 = 0xEEF (MSB set), so addi treats it as -0x111.
    // Upper must be bumped by 1: (0xDEAD + 1) = 0xDEAE, combined = 0xDEADC
    let upper: u32 = 0xDEADC;
    let lower: i32 = 0xEEF_u32 as i32 - 0x1000; // = -0x111

    let program = [encode_lui(upper, 1), encode_itype(lower, 1, 0b000, 1)];
    load_program(&mut cpu, &program);

    cpu.step().unwrap();
    cpu.step().unwrap();

    assert_eq!(cpu.regs[1], 0xDEAD_BEEF, "x1 should hold 0xDEADBEEF");
}

/// AUIPC: compute a PC-relative address and use it as a store/load pointer.
///
/// The program occupies 0x00-0x17 (6 instructions × 4 bytes). We must store
/// data at an address past the program end, so we use offset 0x200 instead of
/// 0x10 to avoid corrupting the instruction stream.
///
///   0x00: auipc x1, 0      ; x1 = 0 (PC-relative base)
///   0x04: addi  x2, x1, 0x200 ; x2 = 0x200 (safe data address)
///   0x08: addi  x3, x0, 0xAB  ; x3 = 0xAB
///   0x0C: sb    x3, 0(x2)     ; bus[0x200] = 0xAB
///   0x10: lb    x4, 0(x2)     ; signed load  -> 0xFFFFFFAB
///   0x14: lbu   x5, 0(x2)     ; unsigned load -> 0x000000AB
#[test]
fn test_auipc_pc_relative_memory_access() {
    let mut cpu = RiscvCpu::new();

    let program = [
        encode_auipc(0, 1),               // 0x00: auipc x1, 0     -> x1=0x00
        encode_itype(0x200, 1, 0b000, 2), // 0x04: addi  x2, x1, 0x200 -> x2=0x200
        encode_itype(0xAB, 0, 0b000, 3),  // 0x08: addi  x3, x0, 0xAB
        encode_stype(0, 3, 2, 0b000),     // 0x0C: sb    x3, 0(x2)
        encode_load(0, 2, 0b000, 4),      // 0x10: lb    x4, 0(x2) signed
        encode_load(0, 2, 0b100, 5),      // 0x14: lbu   x5, 0(x2) unsigned
    ];
    load_program(&mut cpu, &program);

    for _ in 0..6 {
        cpu.step().unwrap();
    }

    assert_eq!(cpu.regs[1], 0x00, "x1 = auipc at PC=0");
    assert_eq!(cpu.regs[2], 0x200, "x2 = data address 0x200");
    assert_eq!(cpu.bus[0x200], 0xAB, "memory byte should be 0xAB");
    assert_eq!(cpu.regs[4], 0xFFFF_FFAB, "lb sign-extends 0xAB");
    assert_eq!(cpu.regs[5], 0xAB, "lbu zero-extends 0xAB");
}

/// JAL used as a loop exit after a BNE countdown loop.
///
///   0x00: addi x1, x0, 3    ; counter = 3
///   0x04: addi x2, x2, 1    ; accumulator++
///   0x08: addi x1, x1, -1   ; counter--
///   0x0C: bne  x1, x0, -8   ; back to 0x04 while x1 != 0
///   0x10: jal  x3, +8       ; jump to 0x18, x3 = 0x14
///   0x14: addi x5, x0, 99   ; skipped
///   0x18: addi x4, x0, 42   ; executes
#[test]
fn test_jal_after_loop() {
    let mut cpu = RiscvCpu::new();
    let program = [
        encode_itype(3, 0, 0b000, 1),  // 0x00: addi x1, x0, 3
        encode_itype(1, 2, 0b000, 2),  // 0x04: addi x2, x2, 1
        encode_itype(-1, 1, 0b000, 1), // 0x08: addi x1, x1, -1
        encode_btype(-8, 1, 0, 0b001), // 0x0C: bne  x1, x0, -8 -> 0x04
        encode_jal(8, 3),              // 0x10: jal  x3, +8 -> 0x18; x3=0x14
        encode_itype(99, 0, 0b000, 5), // 0x14: addi x5, x0, 99 (skipped)
        encode_itype(42, 0, 0b000, 4), // 0x18: addi x4, x0, 42
    ];
    load_program(&mut cpu, &program);

    cpu.step().unwrap(); // setup: addi x1 = 3
    for _ in 0..9 {
        cpu.step().unwrap();
    } // 3 iterations x (addi x2, addi x1, bne)
    cpu.step().unwrap(); // jal x3
    cpu.step().unwrap(); // addi x4 (at 0x18)

    assert_eq!(cpu.regs[1], 0, "counter should be 0 after loop");
    assert_eq!(cpu.regs[2], 3, "accumulator should be 3");
    assert_eq!(cpu.regs[3], 0x14, "x3 = link address 0x14");
    assert_eq!(cpu.regs[5], 0, "x5 should be 0 (skipped)");
    assert_eq!(cpu.regs[4], 42, "x4 = 42");
}
