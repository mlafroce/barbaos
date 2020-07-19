// mem.S
// En virt.lds definimos símbolos del linker con ubicaciones de la memoria
// que representan los límites de nuestras secciones.
// El problema es que si queremos usar esta información, nuestro dato es
// la dirección del puntero y no una variable en sí.
// Definimos constantes globales con la dirección del puntero definido por el linker.
// En nuestro módulo de rust accedemos a ellas utilizando `extern "C"`

#include "generic.h"

.section .rodata
.global TEXT_START
TEXT_START: .word __text_start

.global TEXT_END
TEXT_END: .word __text_end

.global DATA_START
DATA_START: .word __data_start

.global DATA_END
DATA_END: .word __data_end

.global RODATA_START
RODATA_START: .word __rodata_start

.global RODATA_END
RODATA_END: .word __rodata_end

.global BSS_START
BSS_START: .word __bss_start

.global BSS_END
BSS_END: .word __bss_end

.global KERNEL_STACK_START
KERNEL_STACK_START: .word __bss_end

.global KERNEL_STACK_END
KERNEL_STACK_END: .word  __bss_end + 0x80000

.global HEAP_START
HEAP_START: .word __bss_end + 0x80000

// Esta variable se inicializa al inicio del kernel
.global HEAP_SIZE
HEAP_SIZE: .word 0
