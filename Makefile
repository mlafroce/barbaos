RASPI2B_TARGET=armv7a-none-eabi
RISCV_TARGET=riscv64gc-unknown-none-elf

raspi2b:
	cargo run --target $(RASPI2B_TARGET)

riscv:
	cargo run --target $(RISCV_TARGET)

.PHONY: raspi2b riscv
