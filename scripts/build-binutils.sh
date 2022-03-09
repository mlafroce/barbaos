mkdir -p barbaos-binutils-gdb/build
cd barbaos-binutils-gdb/build
../configure \
   --target=riscv64-barbaos \
   --prefix=/opt/barbaos \
   --with-sysroot \
   --disable-werror
