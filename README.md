# riscv-emulator-rust
Building a 32-bit RISC-V (RV32I) emulator from scratch to learn Rust. I'm currently a 3rd-year computer engineering student at SFU, so this is basically a way to take what I'm learning in my labs and see how it works on the software side without dealing with VHDL headaches.

The goal is to go from basic instruction decoding to eventually booting a minimal kernel or some compiled C code.

# Current Progress: Phase 1
I'm currently working through the base integer instructions. All the I-Type arithmetic and logic instructions and R-Type are implemented and verified with unit tests.

## The Roadmap
* Phase 1: The Core (In Progress)

    [x] I-Type decoding (ADDI, SLTI, etc.)

    [x] Shift logic (SLLI, SRLI, SRAI)

    [x] R-Type instructions (ADD, SUB, XOR, etc.)

    [x] Branching logic (BEQ, BNE, etc.)

* Phase 2: Memory & Loads

    [ ] Setup a proper memory bus (Vec<u8>)

    [ ] Load/Store instructions (LW, SW, LB, SB)

    [ ] Handle sign-extension for sub-word loads

* Phase 3: The Runner

    [ ] Fetch-Decode-Execute loop

    [ ] Support for loading raw binary files

    [ ] Basic CLI for stepping through code

* Phase 4: System Level

    [ ] Control & Status Registers (CSRs)

    [ ] Exception handling and ECALLs

    [ ] Virtual UART for terminal output (MMIO)