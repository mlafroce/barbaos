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
