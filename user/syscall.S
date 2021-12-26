.option norvc // Deshabilita compresión de instrucciones
.section .data

.section .text
.global call_syscall
.type   call_syscall, @function

call_syscall:
  ecall
  ret
