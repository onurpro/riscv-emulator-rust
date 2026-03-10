#!/bin/bash

cd "$(dirname "$0")"

mkdir -p bin

riscv64-unknown-elf-as -march=rv32i -mabi=ilp32 -o temp.o asm/test.s
riscv64-unknown-elf-objcopy -O binary temp.o bin/test.bin

rm temp.o

echo "Successfully built programs/bin/test.bin"