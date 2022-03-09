mkdir -p barbaos-gcc/build
cd barbaos-gcc/build
../configure \
    --target=riscv64-barbaos \
    --prefix=/opt/barbaos \
    --enable-languages=c \
    --with-newlib \
    --disable-shared \
    --disable-threads \
    --disable-libssp
