ARCH=riscv64-unknown-elf
CC=$(ARCH)-gcc

%.o : %.S
	$(CC) $< -c

%.o : %.c
	$(CC) $< -c

hello: hello.o syscall.o
	$(CC) -nostdlib -Wl,-Ttext=0x20000000 $^ -o $@

clean:
	$(RM) hello *.o

.PHONY: clean
