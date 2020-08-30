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
  li      t0, (0b11 << 11)
  csrw    mstatus, t0
  // Deshabilito interrupciones durante kinit
  csrw    mie, zero
  // Cargo dirección de memoria de kinit
  la      t1, kinit
  // Valor de retorno al hacer mret (retorno de excepción)
  csrw    mepc, t1
  // Configuro la dirección del vector de traps
  la      t2, m_trap_vector
  csrw    mtvec, t2
  // Interrupciones habilitadas:
  // 1 << 3: software interrupts `irq_software_i`
  // 1 << 7: timer interrupts `irq_timer_i`
  // 1 << 11: external interrupts `irq_extenal_i`
  li      t3, (1 << 3) | (1 << 7) | (1 << 11)
  csrw    mie, t3
  // Ya configuramos las interrupciones, ahora podemos salir hacia el final de este código.
  // Configuramos la dirección de salida en `ra`, cuando salga va a estar en modo supervisor.
  la      ra, after_kinit
  // mret actualiza `mstatus` y sale de una excepción. En nuestro caso, asigna `ra` a nuestro program counter
  mret
after_kinit:

  // Ya configuramos las interrupciones, ahora podemos salir hacia el final de este código.
  // Una vez que termina el kinit, vengo para acá
  // armo el registro `sstatus` (supervisor status)
  // 1 << 8 : Supervisor's previous protection mode -> 1 (SPP=1 [Supervisor]).
  // 1 << 5 : Supervisor's previous interrupt-enable bit -> 1 (SPIE=1 [Enabled]).
  // 1 << 1 : Supervisor's interrupt-enable bit -> 1 después de sret.
  // armamos el estado "previo" que es el que sret va a restaurar
  // li      t0, (1 << 8) | (1 << 5)
  li      t0, (0b00 << 11) | (1 << 7) | (1 << 5)
  csrw    mstatus, t0
  // mideleg (Machine Interrupt delegate)
  // Las interrupciones, por defecto, elevan el privilegio a nivel M
  // Delegamos las interrupciones al nivel de supervisor (ver configuración de reg `mie`)
  // 1 << 1: software interrupts supervisor
  // 1 << 5: timer interrupts supervisor
  // 1 << 9: external interrupts supervisor
  li    t2, (1 << 1) | (1 << 5) | (1 << 9)
  csrw  mideleg, t2
  // Habilito las interrupciones en modo Supervisor
  li    t2, 0xaaa
  csrw  mie, t2
  // Configuro el vector de traps. Estos básicamente un puntero a función, sin embargo
  // si los 2 bits menos significativos son 00, todas las excepiones van a base.
  // Si son 01, van a base + 4 x *causa*
  la    t3, m_trap_vector
  csrw  mtvec, t3
  // Ahora que tenemos todo configurado en memoria principal, forzamos una sincronización 
  // de memoria para asegurarnos que el MMU no agarre valores cacheados
  //sfence.vma
  // Salimos en modo Usuario!
  // Ya no vamos más a kmain, ahora vamos al proceso init, que está en a0
  csrw  mepc, a0
  // Y como el stack está mapeado en el rango 1_0000_0000 ~ 1_0000_2000, voy a tener que actualizarlo
  la    ra, loop_forever
  li    sp, 0x100002000
  mret
secondary_harts:
  // Acá van los otros nucleos esperando a ser despertados por una interrupción de soft.
  // A este tipo de interrupciones se lo llama SIPI (Software Intra-Processor Interrupt).
  // Sólo se usa en modo usuario, quizá cambie
loop_forever:
  // Por convención cerramos nuestro kernel con un loop infinito esperando interrupciones
  wfi
  j loop_forever
