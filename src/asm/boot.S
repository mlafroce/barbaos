.option norvc // Deshabilita compresión de instrucciones
.section .data

.section .text.init
.global _start
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
  la      a0, _bss_start
  la      a1, _bss_end
  bgeu    a0, a1, 2f // si BSS.start es igual BSS.end salteo la inicialización
1:
  sd      zero, (a0) // inicializo BSS.start en 0
  addi    a0, a0, 8  // avanzo el puntero
  bltu    a0, a1, 1b // mientras BSS.start != BSS.end repito
2:
  // Cargo dirección del kernel stack (ver virt.lds)
  // El stack crece de arriba hacia abajo, por eso cargo el final
  la      sp, _stack_end
  // Configuramos mstatus: https://ibex-core.readthedocs.io/en/latest/cs_registers.html#machine-status-mstatus
  // Bits 12:11 -> MPP, machine previous privilege. 11 para modo M
  // Bit 7 -> MPIE, interrupciones de máquina previamente habilitadas
  // Bit 3 -> habilita interrupciones de máquina
  li      t0, (0b11 << 11) | (1 << 7) | (1 << 3)
  csrw    mstatus, t0
  // Cargo dirección de memoria de kmain
  la      t1, kmain
  // Valor de retorno al hacer mret (retorno de excepción)
  csrw    mepc, t1
  // Configuro la dirección del vector de traps
  la      t2, m_trap_vector
  csrw    mtvec, t2
  // Interrupciones habilitadas:
  // 1 << 3: software interrupts `irq_software_i`
  // 1 << 7: timer interrupts `irq_timer_i`
  // 1 << 11: externa interrupts `irq_extenal_i`
  li      t3, (1 << 3) | (1 << 7) | (1 << 11)
  csrw    mie, t3
  // Ya configuramos las interrupciones, ahora podemos salir hacia el final de este código.
  // Configuramos la dirección de salida en `ra`
  la      ra, loop_forever
  // mret actualiza `mstatus` y sale de una excepción. En nuestro caso, asigna `ra` a nuestr program counter
  mret
secondary_harts:
  // Acá van los otros nucleos esperando a ser despertados por una interrupción de soft.
  // A este tipo de interrupciones se lo llama SIPI (Software Intra-Processor Interrupt).
  // Sólo se usa en modo usuario, quizá cambie
loop_forever:
  // Por convención cerramos nuestro kernel con un loop infinito esperando interrupciones
  wfi
  j loop_forever
