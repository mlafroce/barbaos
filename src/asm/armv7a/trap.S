.section .text

.globl _exc_vector, "x"

_exc_vector:
  nop  @ reset  // 0x00
  b handle_unsupported  // 0x04
  b handle_unsupported  // 0x08
  b handle_prefetch_abort  // 0x0C
  b handle_unsupported  // 0x10
  nop  // 0x14 reserved
  b handle_irq_wrapper // 0x18
  b handle_unsupported  // 0x1C

handle_irq_wrapper:
	// Interrupt Service Routine.
	// args = 0, pretend = 0, frame = 0
	// frame_needed = 1, uses_anonymous_args = 0
	sub	lr, lr, #4
	push	{{r0, r1, r2, r3, r4, fp, ip, lr}}
	add	fp, sp, #28
	mrs    	r14, SPSR
	bl	handle_irq
	nop
	sub	sp, fp, #28
	@ sp needed
	ldmfd	sp!, {{r0, r1, r2, r3, r4, fp, ip}}
	msr     SPSR_cf, r14
	ldmfd   sp!, {{pc}}^
