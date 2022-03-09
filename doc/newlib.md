# Newlib en RISC-V

Newlib es un conjunto de bibliotecas que nos permite armar una libc pequeña y básica para compilar aplicaciones y bibliotecas en general. Es muy fácil de adaptar a nuestro SO y de instalar.

**Descargar newlib**

```
git clone git://sourceware.org/git/newlib-cygwin.git newlib
cd newlib
```

Compilamos para RISCV

```
./configure --prefix=$BARBAOS_PREFIX --target=riscv64-unknown-elf --disable-multilib --disable-newlib-supplied-syscalls
```

* `$BARBAOS_PREFIX` será el directorio de instalación (por ejemplo, /opt/barbaos
* `--disable-multilib` previene que se compile la biblioteca para cada una de las variantes de risc-v
* `--disable-newlib-supplied-syscalls` deshabilita las llamadas a syscalls de Newlib.

Implementar las siguientes funciones y compilarlas en un archivo barbaos.o

* `_sbrk`

* `_write`

* `_close`

* `_lseek`

* `_read`

* `_fstat`

* `_isatty`

Ejecutar `make install`

### Bugfixes

En los método `__libc_init_array` y `__libc_fini_array` se ejecutan funciones (constructores y destructores) para objetos estáticos.
El problema es que los arrays contienen un valor sentinela nulo. Para evitar la llamada a un puntero inválido, cambiaremos las siguientes lineas:

*libc/misc/init.c*

```
    if (__init_array_start[i] != 0) {
      __init_array_start[i] ();
    }
```

*libc/misc/fini.c*

```
    if (__fini_array_start[i-1] != 0) {
      __fini_array_start[i-1] ();
    }
```


## Instalación de Binutils y GCC para que tomen newlib

Recompilaremos binutils, que es un conjunto de herramientas utilizadas en la compilación.
En esta primera iteración nos limitaremos a cambiar *sysroot*, que es el directorio que tiene la carpeta *include* y *lib* para compilar y enlazar nuestro programa.

Configuramos tanto binutils como gcc con la siguiente configuración

```
./configure --target=riscv64-unknown-elf --prefix=$BARBAOS_PREFIX --with-sysroot=$BARBAOS_PREFIX/sysroot --disable-werror
```

Una vez configurado, compilamos con `make` e instalamos con `make install`

Luego volvemos a la instalación de *newlib*, ya compilado, e instalamos con make install. Luego copiamos el contenido de `/opt/barbaos/riscv64-unknown-elf/include/` en `/opt/barbaos/include`


# References

* LF-OS blog: https://lf-net.org/blog/posts/2020-05-17-clang-osdev/
* LF-OS git: https://praios.lf-net.org/littlefox/llvm-project/-/commit/09104915b63679d27d02b6a0bfb561e2d01efc57
* https://interrupt.memfault.com/blog/boostrapping-libc-with-newlib

## Newlib revisited

```
./configure --prefix=$BARBAOS_PREFIX --target=riscv64-barbaos
```