#include <stdio.h>
#include "syscall.h"

int main() {
    printf("Hola mundo\n");
    call_syscall(SYS_REBOOT, REBOOT_MAGIC_1, REBOOT_MAGIC_2);
    return 0;
}
