#include "syscall.h"

int main() {
	call_arg_3(SYS_WRITE, 1, "Hello\n", 6);
	call_arg_2(SYS_REBOOT, REBOOT_MAGIC_1, REBOOT_MAGIC_2);
	return 0;
}
