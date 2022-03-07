# BarbaOS

*Si he visto más lejos, es poniéndome sobre los hombros de Gigantes*

Un registro de pasos / tutorial basado en el blog de [Stephen Marz](https://osblog.stephenmarz.com) de cómo crear un sistema operativo para la plataforma RISC-V utilizando el lenguaje Rust.

Este OS no pretende ser un *copy paste* del ya implementado por Stephen (el cual considero una excelente guía para este proyecto), sino abordarlo desde un enfoque distinto, con pasos más chicos e intentando hacer uso de un Rust más idiomático.
En paralelo se intentará desarrollar las mismas funcionalidades para Raspberry PI 2, que utiliza la arquitectura ARMv7

¿Por qué RISC-V? Porque es una arquitectura abierta, libre, lo que favorece mucho a su investigación. Sus instrucciones son sencillas y fáciles de decodificar, además es modular, por lo que podemos agregar módulos de instrucciones según vayamos necesitando.

¿Por qué RPi2? Mantener 2 arquitecturas fuerza mayor modularización. RPi2 es económico y poseo algunos físicos si quiero probar por cuenta propia.

¿Por qué Rust? Porque Rust es un lenguaje de bajo nivel, moderno, que le pone énfasis a la seguridad de memoria, concurrencia, etc. Tiene un compilador muy, muy bueno, y una muy buena comunidad.


## Armando el entorno

Lo primero que tenemos que hacer es descargar un toolchain para RISC-V.
Por suerte Rust tiene incorporado el toolchain suyo de forma oficial, lo que nos facilita mucha configuración.
También es necesario instalar QEmu. En ubuntu se puede instalar el paquete `qemu-system-misc` con el comando

~~~{.bash}
sudo apt install qemu-system-misc
~~~

El mismo trae `qemu-system-riscv32` y `qemu-system-riscv64`

Puede ocurrir que la  ubuntu disponible sea un poco vieja (y en algunas versiones presenta bugs importantes). En tal caso recomiendo clonar y compilar desde el repo oficial:	

~~~{.bash}
git clone git://git.qemu-project.org/qemu.git
cd qemu
./configure --target-list=riscv64-softmmu,riscv32-softmmu --prefix=/opt/qemu
~~~

El prefijo `/opt/qemu` es opcional, para no polucionar las carpetas del sistema. Recordar, si se utiliza un prefijo de instalación, agregar al $PATH la ruta de los binarios instalados.

Si se lo desea, se encuentran también las instrucciones para descargar el toolchain de gcc [acá](https://github.com/riscv/riscv-gnu-toolchain)

En este toolchain se puede encontrar herramientas útiles como gdb (necesario para correr las pruebas)

### Tests

Para ejecutar los tests, utilizamos *pytest* y *pygdbmi*. Instalar con el siguiente comando.

~~~{.bash}
pip3 install pytest pygdbmi
~~~


## Kernel vacío

En el primer paso compilamos bastante assembly.
Creamos un archivo de assembly "boot.S", en el que escribimos las instrucciones para que nuestra CPU inicialice, configuramos registros, inicializamos stack, interrupciones etc.

También creamos un script de linkedición. Este script describe las regiones de memoria a utilizar, en particular para correr con *QEMU*
Finalmente hacemos una biblioteca *bare-bones* en rust, que posee un método main vacío a la espera de interrupciones.

Para ejeuctarlo escribimos

~~~{.bash}
cargo run
~~~

La configuración en ".cargo/config" lanzará qemu y correrá nuestro kernel (que no hace nada)

Si queremos ejecutarlo en modo debug, ejecutamos

~~~{.bash}
cargo run -- -s -S
~~~

El primer "-s" es para que se levante un servidor de gdb en el puerto :1234, el segundo, "-S", es para que el kernel no arranque hasta que se le de la orden desde el monitor / cliente gdb.

### Tests con python

Con este kernel vacío lo único que podemos hacer es ejecutarlo. Escribimos un test para probar que podemos levantar una instancia de QEmu y matarla.


## Device tree

RISC-V, al igual que otras arquitecturas de embebidos, utiliza un **Device tree** para brindarle al sistema información sobre el hardware presente.
Esta información de los dispositivos emulados es muy similar al *ACPI* en arquitecturas x86.

El *device tree* se carga en memoria en un formato binario, que podemos guardar en disco utilizando el siguiente comando:

```bash 
qemu-system-riscv64 -machine virt,dumpdtb=qemu-riscv64-virt.dtb
```

Para convertirlo a texto plano usamos el comando `dtc` y escribimos la salida en un archivo *.dts*

```bash
dtc qemu-riscv64-virt.dtb > qemu-riscv64-virt.dts
```

Cuando inspeccionamos este archivo podemos ver muchos datos interesantes, como por ejemplo, ubicación y tamaño de la memoria, cantidad de CPUs, dispositivos conectados y compatibilidad de los mismos.
En particular, el que más nos va a interesar para iniciar es el dispositivo UART, que vemos que es compatible con el modelo "ns16550a".

El dispositivo UART es un *puerto serie* con el que vamos a imprimir por pantalla. 

Si leemos el código fuente del bootloader [OpenSBI](https://github.com/riscv-software-src/opensbi/blob/master/docs/firmware/fw.md), podemos ver que al iniciar el emulador, tenemos en el registro `a1` la dirección del DTB.

Más información sobre los dispositivos que podemos encontrar: https://www.sifive.com/blog/risc-v-qemu-part-1-privileged-isa-hifive1-virtio

### Poweroff y reboot

En el *dts* podemos ver 2 propiedades interesantes: `poweroff` y `reboot`. Vemos que en el caso de *poweroff*, tenemos un valor de `0x5555`, y dice ser compatible con `syscon-poweroff`, entre otros datos.

Más abajo podemos ver el dispositivo `test@1000000`, que casualmente es compatible con *syscon*. Lo que nos permite este dispositivo es escribirle el valor de la acción que queremos realizar: apagar o reiniciar la máquina.

Escribimos una función `shutdown` que nos permita apagar la máquina virtual y salir de qemu.

### Dispositivo UART

Creamos un dispositivo *UART*. Para esto, sabiendo que QEMU designa la posición de memoria *0x1000_0000* para virtualizar un dispositivo NS16550 "instanciamos" uno en nuestro kmain para informar sobre la ejecución del kernel.

En nuestras pruebas en python vamos a agregarle la opción `stdout=PIPE` a nuestro proceso de QEmu para poder leer la salida del kernel y compararla contra nuestras pruebas


## Tests en Rust

Rust tiene un framework de test incorporado al sistema de compilación, sin embargo este necesita de la biblioteca `std` para poder ejecutarse. Por suerte podemos utilizar nuestro propio "framework" de pruebas. Para esto definimos una función como `test_runner`, la cual compilamos únicamente si estamos corriendo tests.

Al declarar nuestro *runner*, se creará automáticamente una función "`main`" que llamará a nuestro wrapper. Podemos renombrar esta función configurando el `reexport_test_harness_main`

Una vez configurados los wrappers de nuestras pruebas, hacemos una llamada al *harness* desde nuestro `kmain`.

Por el momento esto nos puede ayudar a ejecutar tests sencillos, aunque deberá ser remodelado cuando se separe el modo máquina del modo supervisor / usuario.

## Recorrido del DTB

Agregamos en el arranque un parseo básico del *device tree*, con el que imprimimos información básica del CPU y memoria ram.

## Paginas de memoria

Exportamos direcciones de memoria con el script del linkeditor y luego usamos un snippet en assembly para guardar esas direcciones en variables globales. Luego definiremos un tamaño de página de memoria standard, de 4kb, y luego, conociendo la dirección del heap y su tamaño, procedemos a dividir el heap en páginas.

Utilizamos el dispositivo UART para imprimir las direcciones de memoria que identificamos.


### Alloc

En este paso hacemos un alloc primitivo de páginas. Recibo por parámetro la cantidad de páginas que quiero reservar de forma continua.

Iteramos la lista de páginas y buscamos una secuencia de N páginas libres. Si la encontramos, devolvemos el puntero.


### Dealloc

Implementamos un dealloc para liberar la memoria reservada por `alloc`. Hacemos un chequeo básico de no estar liberando un puntero nulo o un double free.


## Global allocator

Rust, de forma similar a C++, maneja sus recursos utilizando el concepto de *Lifetime* y *Scope*.
Cuando declaramos una variable, la misma va a vivir dentor del *scope* en el que fue declarada, y una vez que el scope termina, la variable se "destruye" y se liberan sus recursos.
Rust flexibiliza la restricción del *scope* agregando el concepto de **ownership**. Una variable puede ccambiar su *scope* cambiando su *owner*.

Este manejo de recursos nos ayuda a evitar leaks de memoria y punteros "sueltos". Para evitar el uso de punteros sueltos, usamos estructuras como `Vec` y `Box`

Para poder usar estas estructuras, creamos un **Allocator**

Rust nos exige un allocator global para poder hacer uno particular. Utilizamos el mismo para ambos fines y hacemos un test usando la api (actualmente experimental) de *allocators*.

En Rust 2024 el uso de variables static mut está deprecado, por lo que usamos un UnsafeCell para tener nuestra clase con mutabilidad interna.


## Interrupciones

Lo primero que hacemos es completar nuestra función `asm_trap_vector`, en la que ahora hacemos una copia de todos los registros y el registro de control y status (CSR) `mscratch`. Una vez que copiamos al stack todos nuestros registros, saltamos a la función `m_trap_handler` para analizar el motivo de la interrupción.
Por el momento sólo adelantamos el *program counter* o ejecutamos un *panic!* según el motivo. Una vez que imprimimos el motivo de nuestra interrupción, volvemos a la ejecución

**IMPORTANTE**: hay que tener en cuenta que, si estamos sobre una instrucción comprimida, al sumar 4 al *program counter* podemos caer sobre una instrucción no alineada. Esto rompería con la ejecución de nuestro sistema. Sin embargo, más adelante veremos que, en vez de adelantar el *program counter*, podemos llamar a funciones más específicas para este tipo de situaciones.


### Interrupciones del timer

Para que el timer interno lance interrupciones tenemos que usar dos posiciones de memoria:

* `mtime`: dirección del reloj interno del sistema.

* `mtimecmp`: dirección con un tiempo que será comparado contra `mtime`, cuando el tiempo sea el mismo, se lanza una interrupción.

Para lanzar una interrupción periódicamente, leemos el contenido de `mtime`, le sumamos un valor *delta* y lo almacenamos en la dirección `mtimecmp`.

Estas interrupciones seran de utilidad para implementar multi-tasking


### Interrupciones externas

RISCV utiliza un controlador llamado **PLIC** (*Platform level interrupt controller*). Este controlador se utiliza para atender interrupciones externas. Como utilizamos QEmu sólo nos interesa las interrupciones de UART. Configuramos el id correspondiente al UART realizamos una lectura del dispositivo cada vez que recibimos un llamada.

Ahora que nuestra entrada depende de una interrupción, podemos quitar el *busy loop* de `kmain` y llamar a `abort`. Esta función ejecuta una instrucción *wait for interrupt*. Podemos ver como baja enormemente el consumo de CPU con la nueva forma de ingresar caracteres.


## MMU (sin page faults)

RISC-V es una arquitectura modular, algunas de sus implementaciones son muy similares a las de un microcontrolador: muy pocas instrucciones, sin instrucciones de privilegio y sin MMU. Por este motivo, el MMU es un hardware externo al procesador.
Cuando el sistema corre en el anillo de seguridad 1 (modo *M*), el de máquina, la *MMU* está desactivada y hacemos uso directo de la memoria.
Cuando pasamos al anillo 2, el de supervisor, activamos la *MMU*.

La *MMU* se configura con el registro **SATP** (Supervisor Address Translation and Protection). Este registro tiene 3 campos:

* **MODE**: que define el tipo de transformación (0 si se usa memoria física, otros valores según si es RISCV-32 o RISCV-64).

* **ASID**: utilizado para asociar un espacio de memoria (*address space*) a un proceso. Podemos elegir 0 y recargar toda la TLB, o usar algo único como el PID para solo recargar si es necesario.

* **PPN**: *Physical page number*, la *dirección* de la página donde va a estar alojada nuestra TLB. Se le quitan los 12 primeros bits (porque las páginas son de 2^12 bytes).
  

## Modo supervisor

Como indicamos, el módo *Máquina* utiliza memoria física, si queremos usar memoria virtual necesitamos pasar al modo *Supervisor* o *Usuario*. Para esto convertimos nuestra función *kmain* en *kinit*, donde se inicializan los dispositivos, paginación de memoria y memoria virtual. Una vez inicializado devolvemos el valor del registro `satp`. Este registro posee la ubicación (alineada a una página de memoria) de la raiz de nuestro **TLB**, como se describió anteriormente. Se configura este registro y se hace un retorno de interrupción para pasar al modo supervisor.

Una vez configurada la MMU, este modo *Supervisor* puede trabajar con memoria virtual. Como las configuraciones trabajadas hasta ahora fueron realizadas en modo *máquina*, debemos configurar la tabla de mapeo de memoria de las regiones accedidas.
Es necesario configurar la TLB para acceder a la memoria con las instrucciones, dispositivos, etc. De lo contrario sucederá un *Page Fault*, y al no tener interrupciones configuradas, nuestro kernel quedará loopeando infinitamente.

También debemos recordar configurar la Physical Memory Protection (PMP). Esta protección nos permite poner restricciones a ciertas regiones de memoria incluso en modo M.
Luego de la versión 6.0 de Qemu, es mandatoria la configuración de estos registros.

### Memoria virtual

Agregamos la clase `MapTable`, que tendrá la lógica de como llenar las tablas de paginación de RISC-V. Esto es necesario para salir del modo *máquina*


## Dispositivos E/S

Dado que estamos desarrollando sobre **QEMU**, escribimos un driver básico para dispositivos de entrada y salida. La meta es crear un driver para poder leer un "disco externo" y poder comunicar nuestro S.O. con el host.

Podemos observar las direcciones de los dispositivos disponibles ejecutando `ctrl + a, c` en Qemu para abrir la consola, y luego ejecutar `info qtree`

Utilizamos el protocolo de VirtIO. Escaneamos las direcciones de memoria de estos dispositivos (`0x1000_1000` a `0x1000_8000`) e identificamos dispositivos montados en el emulador.

En particular nos interesa saber que su *magic number* es `virt` y nos interesan los dispositivos de tipo bloque (es decir, 2).

### Inicialización de dispositivos

De acuerdo a la sección 3.1 de la especificación de VirtIO, para configurar un dispositivo tenemos los siguientes pasos:

1. Escribir el status `RESET`
2. Escribir el status `ACKNOWLEDGE`
3. Escribir el status `DRIVER`
4. Leer bits de features, validar cuáles pueden ser aceptados. Ofrecer mis features escribiendo en el registro de guest features
5. Escribir el status `FEATURES_OK` para contrastar los features del host con el guest.
6. Leer y validar que el status siga siendo `FEATURES_OK`.
7. Configuración especifica del driver
8. Escribir el estado `DRIVER_OK`


### Lectura de bloques

Para leer (o escribir) un bloque de datos, encolamos un `Request`
Creamos una instancia de esta estrutura, como está descripto en la sección *Device Operation*. Pasamos la dirección de la misma al `addr` del Descriptor, y lo asignamos en la VirtQueue.


## Procesos

¡Nos preparamos para el modo usuario!

Creamos un struct `Process` con componentes básicos de un proceso: un frame con el estado de los registros y el *SATP*, stack, entre otros. El stack del proceso, un program counter inicial del proceso, el id del proceso (pid), una tabla de paginación para mapear la memoria, y el estado del proceso.

Creamos un frame para correr un proceso, el proceso `init()`. Por ahora, como no tenemos nada mapeado, nos limitamos a hacer un loop vacío. Al proceso le pasamos la dirección de una función de rust nuestra, pero mapeada a una dirección de memoria virtual elegida por nosotros.
Quitamos la función `kmain`, ya que ahora vamos a iniciar el proceso en modo usuario. 

En `trap.rs` agregamos que, antes de atender una interrupción, que actualice el stack pointer, al del stack.


## Cargar un ELF

En los sistemas unix, el formato ELF (*Executable and Linkable Format*) es el formato más utilizado para almacenar ejecutables. 
Vamos a crear un ELF, que cargaremos en memoria, para ejecutar en modo usuario.

Lo primero que vamos a crear es una carpeta `user` para nuestras aplicaciones, en esta carpeta vamos a incluir código y scripts de las aplicaciones a correr a nivel usuario.

Dado que la ABI de C es más simple y sus ejecutables tienen menor overhead, empezamos haciendo un "Hola mundo" muy crudo utilizando las syscalls existentes.

### Pasos previos a armar una *libc*

En los sistemas tipo unix existe una biblioteca llamada *libc*, que, en conjunto con otras importantes como *libm* o *libpthread*, implementan lo que se conoce como la **biblioteca standard de C**.

Esta biblioteca posee funciones muy comunes como "printf" o "strcpy". Algunas de estas funciones casi se mapean de forma directa contra las syscalls.

Existen varias implementaciones de la biblioteca standard, como la GNU libc (glibc), musl, uClibc, Newlib, o incluso uno mismo puede crearse su propia libc.

Per antes de aventurar en el mundo de las bibliotecas, hay que armar una interfaz de syscalls para comunicarse correctamente con el kernel

Refactorizamos el módulo de syscalls: creamos un archivo `src/system/syscall/sys.rs` que será la raiz de nuestra biblioteca libsys.
Separamos nuestro archivo syscall.rs en `api.rs`, que contiene declaraciones de constantes y las llamadas a syscalls.

Luego generamos la biblioteca estática libsys, ejecutando

```bash
 mv libsys.a user/
```

Generamos una API para código C, que posteriormente podamos usar con alguna `libc` ya existente.

```bash
cbindgen src/system/syscall/sys.rs -o user/syscall.h -c cbindgen.toml
```

Es posible que en el header generado no se encuentre el tipo `uintptr_t`, para lo que hacemos un
```c
typedef unsigned long int uintptr_t;
```

También puede ser útil eliminar las llaves del bloque `extern "C"`

### Hello world desde un ELF externo

Creamos un archivo hello.c con el siguiente contenido:
```c
#include "syscall.h"

int main() {
	call_arg_3(SYS_WRITE, 1, "Hello\n", 6);
	call_arg_2(SYS_REBOOT, REBOOT_MAGIC_1, REBOOT_MAGIC_2);
	return 0;
}
```

Lo compilamos sin bibliotecas standard, sólo linkeamos contra nuestra *libsys*, para ejecutar syscalls

```bash
riscv64-unknown-elf-gcc -nostdlib -Wl,-Ttext=0x20000000 hello.c libsys.a -o hello
```

Finalmente modificamos nuestro proceso init para que inicie desde otra posición de memoria.
En el siguiente comando, indicamos que vamos a usar un "loader" para cargar en memoria el contenido del binario `user/hello`

```
cargo run -- -device loader,file=./user/hello,addr=0x82000000,force-raw=true
```

*Nota*: además de los pasos anteriores, se incluyó una interfaz hecha en C para simplificar el analisis del ELF

**Importante**: el formato ELF describe cómo se distribuyen las secciones en memoria, el valor inicial del program counter, entre otros datos.
Dado que estamos ignorando este contenido, es importante respetar el orden de los objetos compilados, ya que esto determina cómo se ordenan en la sección ".text" del ELF. Lo más correcto es utilizar un cargador de archivos ELF.

## Cargador de archivos ELF

Para ejecutar ELFs más complejos correctamente y sin depender de que el azar ordene nuestras funciones de la forma correcta, hacemos un
cargador de  archivos ELF básico. Este cargador lee los headers del archivo para determinar el *entry point* y las secciones que componen
nuestra aplicación. Cargamos todas las secciones a un conjunto de páginas reservado por nosotros y paginamos con permisos de ejecución o
escritura según corresponda. La validación de errores es mínima, se usará un loeader más completo cuando se disponga de una toolchain completa.
