#ifndef SYSCALL_H
#define SYSCALL_H

typedef unsigned long int uintptr_t;

static const uintptr_t REBOOT_MAGIC_1 = 318839184;

static const uintptr_t REBOOT_MAGIC_2 = 3402301098;

static const uintptr_t SYS_WRITE = 1;

static const uintptr_t SYS_REBOOT = 48;

void call_syscall(int id, ...);

void call_arg_0(uintptr_t syscall_id);

void call_arg_1(uintptr_t syscall_id, uintptr_t arg0);

void call_arg_2(uintptr_t syscall_id, uintptr_t arg0, uintptr_t arg1);

void call_arg_3(uintptr_t syscall_id, uintptr_t arg0, uintptr_t arg1, uintptr_t arg2);

#endif
