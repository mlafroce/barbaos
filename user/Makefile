ARCH=riscv64-unknown-elf
CC=$(ARCH)-gcc
NEWLIB_ROOT=$(HOME)/misc/osdev/newlib/newlib-cygwin
NEWLIB_INCLUDE=$(NEWLIB_ROOT)/newlib/libc/include
NEWLIB_LIBRARIES=$(NEWLIB_ROOT)/riscv64-unknown-elf/rv64imafdc/lp64d/newlib
LIBGLOSS_LIBRARIES=$(NEWLIB_ROOT)/riscv64-unknown-elf/rv64imafdc/lp64d/libgloss/riscv

all: hello hello-newlib

%.o : %.S
	$(CC) $< -c

%.o : %.c
	$(CC) -I$(NEWLIB_INCLUDE) -L$(NEWLIB_LIBRARIES) $< -c

hello: hello.o syscall.o
	$(CC) -nostdlib -Wl,-Ttext=0x20000000 $^ -o $@

hello-newlib: syscall.o hello-newlib.o api.o
	$(CC) -nostartfiles -fno-use-cxa-atexit -L$(LIBGLOSS_LIBRARIES) -L$(NEWLIB_LIBRARIES) -Wl,-Ttext=0x20000000 $(LIBGLOSS_LIBRARIES)/crt0.o $^ -o $@

clean:
	$(RM) hello hello-newlib *.o

.PHONY: clean
