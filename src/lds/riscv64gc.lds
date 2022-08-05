/*
 virt.lds
 Linker script for outputting to RISC-V QEMU "virt" machine.
 Stephen Marz
 6 October 2019
*/

/*
  riscv is the name of the architecture that the linker understands
  for any RISC-V target (64-bit or 32-bit).

  We will further refine this by using -mabi=lp64 and -march=rv64gc
*/
OUTPUT_ARCH( "riscv" )

/*
We're setting our entry point to a symbol
called _start which is inside of boot.S. This
essentially stores the address of _start as the
"entry point", or where CPU instructions should start
executing.

In the rest of this script, we are going to place _start
right at the beginning of 0x8000_0000 because this is where
the virtual machine and many RISC-V boards will start executing.
*/
ENTRY( _start )

/*
PHDRS is short for "program headers", which we specify three here:
text - CPU instructions (executable sections)
data - Global, initialized variables
bss  - Global, uninitialized variables (all will be set to 0 by boot.S)

The command PT_LOAD tells the linker that these sections will be loaded
from the file into memory.

We can actually stuff all of these into a single program header, but by
splitting it up into three, we can actually use the other PT_* commands
such as PT_DYNAMIC, PT_INTERP, PT_NULL to tell the linker where to find
additional information.

However, for our purposes, every section will be loaded from the program
headers.
*/
PHDRS
{
  text PT_LOAD;
  data PT_LOAD;
  bss PT_LOAD;
}

BASE_ADDR = 0x80000000;

/*
We are now going to organize the memory based on which
section it is in. In assembly, we can change the section
with the ".section" directive. However, in C++ and Rust,
CPU instructions go into text, global constants go into
rodata, global initialized variables go into data, and
global uninitialized variables go into bss.
*/
SECTIONS
{
  /*
    The first part of our RAM layout will be the text section.
	Since our CPU instructions are here, and our memory starts at
	0x8000_0000, we need our entry point to line up here.
  */
  . = BASE_ADDR;
  .text : {
    /*
      PROVIDE allows me to access a symbol called _text_start so
      I know where the text section starts in the operating system.
      This should not move, but it is here for convenience.
      The period '.' tells the linker to set _text_start to the
      CURRENT location ('.' = current memory location). This current
      memory location moves as we add things.
	*/

    PROVIDE(_text_start = .);
      /*
      We are going to layout all text sections here, starting with
	  .text.init. The asterisk in front of the parentheses means to match
	  the .text.init section of ANY object file. Otherwise, we can specify
	  which object file should contain the .text.init section, for example,
	  boot.o(.text.init) would specifically put the .text.init section of
	  our bootloader here.

	  Because we might want to change the name of our files, we'll leave it
	  with a *.

	  Inside the parentheses is the name of the section. I created my own
	  called .text.init to make 100% sure that the _start is put right at the
	  beginning. The linker will lay this out in the order it receives it:

	  .text.init first
	  all .text sections next
	  any .text.* sections last

	  .text.* means to match anything after .text. If we didn't already specify
	  .text.init, this would've matched here. The assembler and linker can place
	  things in "special" text sections, so we match any we might come across here.
	*/
    *(.text.init) *(.text .text.*)

	/*
	  Again, with PROVIDE, we're providing a readable symbol called _text_end, which is
	  set to the memory address AFTER .text.init, .text, and .text.*'s have been added.
	*/
    PROVIDE(_text_end = .);
  }
   /*
     The global pointer allows the linker to position global variables and constants into
	 independent positions relative to the gp (global pointer) register. The globals start
	 after the text sections and are only relevant to the rodata, data, and bss sections.
   */
   PROVIDE(_global_pointer = .);
   /*
     Most compilers create a rodata (read only data) section for global constants. However,
	 we're going to place ours in the text section. We can actually put this in :data, but
	 since the .text section is read-only, we can place it there.

	 NOTE: This doesn't actually do anything, yet. The actual "protection" cannot be done
	 at link time. Instead, when we program the memory management unit (MMU), we will be
	 able to choose which bits (R=read, W=write, X=execute) we want each memory segment
	 to be able to do.
   */
  .rodata : {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);
	/*
	   Again, we're placing the rodata section in the memory segment "ram" and we're putting
	   it in the :text program header. We don't have one for rodata anyway.
	*/
  }

  .data : {
	/*
	   . = ALIGN(4096) tells the linker to align the current memory location (which is
	   0x8000_0000 + text section + rodata section) to 4096 bytes. This is because our paging
	   system's resolution is 4,096 bytes or 4 KiB.
	*/
    . = ALIGN(4096);
    PROVIDE(_data_start = .);
	/*
	   sdata and data are essentially the same thing. However, compilers usually use the
	   sdata sections for shorter, quicker loading sections. So, usually critical data
	   is loaded there. However, we're loading all of this in one fell swoop.
	   So, we're looking to put all of the following sections under the umbrella .data:
	   .sdata
	   .sdata.[anything]
	   .data
	   .data.[anything]

	   ...in that order.
	*/
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  }

  .bss : {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  }
}
