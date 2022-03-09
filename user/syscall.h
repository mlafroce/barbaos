#ifndef SYSCALL_H
#define SYSCALL_H

typedef unsigned long int uintptr_t;

static const uintptr_t REBOOT_MAGIC_1 = 318839184;

static const uintptr_t REBOOT_MAGIC_2 = 3402301098;

static const uintptr_t SYS_WRITE = 1;

static const uintptr_t SYS_BRK = 12;

static const uintptr_t SYS_REBOOT = 48;

void call_syscall(int id, ...);

#endif
