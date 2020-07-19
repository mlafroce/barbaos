# Crear disco

~~~
fallocate -l 32M hdd.dsk
~~~

# Debugger

* Ejecutar SO en una termianl: `cargo run -- -s -S`

* Ejecutar `riscv64-unknown-elf-gdb os.elf`, `target remote :1234`
