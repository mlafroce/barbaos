ARCH=riscv64-unknown-elf
CC=$(ARCH)-gcc
NEWLIB_ROOT=$(HOME)/misc/osdev/newlib/newlib-cygwin
NEWLIB_INCLUDE=$(NEWLIB_ROOT)/newlib/libc/include
NEWLIB_LIBRARIES=$(NEWLIB_ROOT)/build/riscv64-unknown-elf/newlib
LIBGLOSS_LIBRARIES=$(NEWLIB_ROOT)/build/riscv64-unknown-elf/libgloss/riscv

all: hello hello-newlib

%.o : %.S
	$(CC) $< -c

%.o : %.c
	$(CC) -I$(NEWLIB_INCLUDE) $< -c

hello: hello.o syscall.o
	$(CC) -nostdlib -Wl,-Ttext=0x20000000 $^ -o $@

hello-newlib: syscall.o hello-newlib.o api.o
	$(CC) -B$(LIBGLOSS_LIBRARIES) -L$(NEWLIB_LIBRARIES) $^ -o $@

clean:
	$(RM) hello hello-newlib *.o

.PHONY: clean
