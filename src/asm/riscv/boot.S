.option norvc // Deshabilita compresión de instrucciones

.equ KERNEL_STACK_SIZE, 0x80000

.section .data

.section .text.init
.global _start

// Inicializar registros y frames de la CPU
// QEmu nos indica cuál es la dirección de inicio de la memoria en `a0` y la dirección del DTB en `a1`
// Más adelante utilizaremos el DTB para obtener información del hardware disponible
// En QEmu 5.0 la dirección del DTB coincide con el final del heap
_start: 
  // Lee el ID de la cpu
  csrr    t0, mhartid
  // Si es distinto de 0, se va al final
  bnez    t0, secondary_harts
  // Limpia SATP (usado por la MMU)
  csrw    satp, zero
  // Acá le pido al compilador que no haga optimizaciones y guarde en gp
  // la dirección del $pc actual
.option push
.option norelax
  la      gp, _global_pointer
.option pop
  // Sección BSS, con variables estáticas generalmente no inicializadas
  la      t0, _bss_start
  la      t1, _bss_end
  bgeu    t0, t1, 2f // si BSS.start es igual BSS.end salteo la inicialización
1:
  sd      zero, (a0) // inicializo BSS.start en 0
  addi    t0, t0, 8  // avanzo el puntero
  bltu    t0, t1, 1b // mientras BSS.start != BSS.end repito
2:
  // Agrego un offset para el stack pointer
  li        t0, KERNEL_STACK_SIZE
  add       sp, t1, t0
  // Configuración inicial del modo máquina
  j         machine_mode_init
secondary_harts:
  // Acá van los otros nucleos esperando a ser despertados por una interrupción de soft.
  // A este tipo de interrupciones se lo llama SIPI (Software Intra-Processor Interrupt).
  // Sólo se usa en modo usuario, quizá cambie
loop_forever:
  // Por convención cerramos nuestro kernel con un loop infinito esperando interrupciones
  wfi
  j loop_forever
