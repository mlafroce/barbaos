// trap.S
// Trap handler and global context
// Steve Operating System
// Stephen Marz
// 24 February 2019
.option norvc
.altmacro
.set NUM_GP_REGS, 32  // Cantidad de registros de uso general
.set NUM_FP_REGS, 32  // Cantidad de registros de punto flotante
.set REG_SIZE, 8      // Tamaño (en bytes) de un registro 

// Macros para guardar registros
// Utilizan a t6 como base del stack por defecto
.macro save_gp i, basereg=t6
    sd  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
    ld  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
    fsd f\i, ((NUM_FP_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
    fld f\i, ((NUM_FP_REGS+(\i))*REG_SIZE)(\basereg)
.endm


// Assembly-level trap handler.
// m_trap_vector va a ser nuestra "función" para atender
// interrupciones en modo máquina
.section .text
.global m_trap_vector
// Debe estar alineado ya que los 2 bits menos significativos
// de mtvec no forman parte de la dirección
.align 4 
m_trap_vector:
    // Copiamos todos los registros del frame actual, primero copiamos
    // mscratch como si fuera un stack pointer de esta función
    csrrw   t6, mscratch, t6
    // csrrw hace un swap atómico entre t6 y mscratch, sin destruir nada
    // Nuestro frame va a contar con la siguiente información:
    // 32 gp regs       0
    // 32 fp regs       256
    // SATP register    512
    // Trap stack       520
    // CPU HARTID       528
    // El tutorial utiliza t6 porque es el último registro (x31), pero en
    // otros sitios también se sugiere usar el stack pointer
    // copiamos los registros 0-30
    .set    i, 1
    .rept   30
        save_gp %i
        .set    i, i+1
    .endr

    // en el t6 tenemos a mscratch, lo pasamos a t5
    mv      t5, t6
    // Ahora hacemos una lectura atómica del t6 intercambiado previamente, restaurandolo
    csrr    t6, mscratch
    // 31 (que es t6) se guarda
    save_gp 31, t5

    // Pongo en mscratch su valor original
    csrw    mscratch, t5

    // Coloco los parámetros para llamar a m_trap_handler
    csrr    a0, mepc
    csrr    a1, mtval
    csrr    a2, mcause
    csrr    a3, mhartid
    csrr    a4, mstatus
    mv      a5, t5

    // Uso stack global del kernel
    la      t0, KERNEL_STACK_END
    ld      sp, 0(t0)
    call    m_trap_handler

    // `m_trap_handler` devuelve un entero `usize`, el mismo se encuentra
    // en `a0`, y contiene el *program counter* actualizado
    csrw    mepc, a0

    // Ahora copio el `mscratch` nuevamente a t6, para poder volver a usarlo
    // como stack pointer
    csrr    t6, mscratch

    // Restauro _todos_ los registros, incluyendo t6
    .set    i, 1
    .rept   31
        load_gp %i
        .set    i, i+1
    .endr
    mret
