# Scripts

* *download.sh*: downloads barbaos binutils and gcc. Use --zip for *zip* file or --git for git repo cloning.

* *build-binutils.sh*: compile binutils + gdb for riscv64-barbaos

* *build-gcc-stg1.sh*: compile gcc for riscv64-barbaos, freestanding. Depends on binutils.

* *build-newlib.sh*: compile newlib for riscv64-barbaos, using previously compiled gcc.

* *build-gcc-stg2.sh*: compile gcc for riscv64-barbaos with newlib support.

## Recommended

* *tools.sh*: download and compile automake and autoconf versions used when developing binutils and gcc

# Dependencies

To build binutils and gcc the following packages must be present:

build-essentials
zip
autoconf
automake
libgmp-dev
libmpfr-dev
libmpc-dev

## Recommended for binutils-gdb

bison
flex
texinfo
