#include <stdio.h>
#include "syscall.h"

int main() {
	printf("Hola mundo\n");
    call_arg_2(SYS_REBOOT, REBOOT_MAGIC_1, REBOOT_MAGIC_2);
    return 0;
}

void call_arg_1(uintptr_t syscall_id, uintptr_t arg0) {
    call_syscall(syscall_id, arg0);
}

void call_arg_2(uintptr_t syscall_id, uintptr_t arg0, uintptr_t arg1) {
    call_syscall(syscall_id, arg0, arg1);
}

void call_arg_3(uintptr_t syscall_id, uintptr_t arg0, uintptr_t arg1, uintptr_t arg2) {
    call_syscall(syscall_id, arg0, arg1, arg2);
}
