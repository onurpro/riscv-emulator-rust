use riscv_emulator_rust::RiscvCpu;
use std::fs;
use std::process;

fn main() {
    // Setup CPU
    let mut cpu = RiscvCpu::new(1024 * 64);

    //Setup program
    // let program: Vec<u32> = vec![
    //     0x00a00093, // li x1, 10
    //     0x01400113, // li x2, 20
    //     0x002081b3, // add x3, x1, x2
    //     0x00100073, // EBREAK
    // ];

    // for (i, instr) in program.iter().enumerate() {
    //     let bytes = instr.to_le_bytes();
    //     let start = i * 4;
    //     cpu.bus[start..start + 4].copy_from_slice(&bytes);
    // }

    let program = fs::read("programs/bin/test.bin").expect("Failed");

    cpu.bus[0..program.len()].copy_from_slice(&program);

    loop {
        match cpu.step() {
            Ok(_) => {
                println!("Executed PC: {:#x}", cpu.pc);
                cpu.dump_registers();
            }
            Err(e) => {
                println!("\n[CPU HALTED]: {}", e);
                process::exit(1);
            }
        }
    }
}
