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

### Tests

Con este kernel vacío lo único que podemos hacer es ejecutarlo. Escribimos un test para probar que podemos levantar una instancia de QEmu y matarla.
