.equ KERNEL_STACK_SIZE, 0x100000

.section .data

.section .text.init

// Make _start global.
.global _start
    .org 0x8000
// Entry point for the kernel.
// r15 -> should begin execution at 0x8000.
// r0 -> 0x00000000
// r1 -> 0x00000C42 - machine id
// r2 -> 0x00000100 - start of ATAGS
// preserve these registers as argument for kernel_main
_start:
	// ARMv7 uses "coprocessors" to store system data
	mrc p15, 0, r5, c0, c0, 5
	and r5, r5, #3
	cmp r5, #0
	bne halt

	// Setup the stack.
	ldr r5, =_start
	mov sp, r5

	// Clear out bss.
	ldr r4, =__bss_start
	ldr r9, =__bss_end
	mov r5, #0
	mov r6, #0
	mov r7, #0
	mov r8, #0
	b       2f

1:
	// store multiple at r4.
	stmia r4!, {{r5-r8}}

	// If we are still below bss_end, loop.
2:
	cmp r4, r9
	blo 1b

    ldr	r4, = _exc_vector
    mcr	p15, 0, r4, c12, c0, 0	// Secure or Non-secure VBAR
    mcr	p15, 0, r4, c12, c0, 1	// Monitor Vector Base Address Register

    mrs	r4, spsr
    bic	r5, r4, #0x0000001f
    // Setup the stack IRQ mode.
    orr	r5, r5, #0x12
    msr	cpsr_c, r5
    ldr	sp, =_irq_stack+4096

    // Return to the original mode.
    msr cpsr_c, r4

	// Call machine_init
	ldr r3, =machine_init
	blx r3

	// halt
halt:
	wfi
	b halt
