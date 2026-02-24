use riscv_emulator_rust::RiscvCpu;

fn main() {
    let mut cpu = RiscvCpu::new();

    let test_instr: u32 = 0x00100513;
    let bytes = test_instr.to_le_bytes();
    cpu.bus[0..4].copy_from_slice(&bytes);

    cpu.step();
    // cpu.step();
}