# BarbaOS

*Si he visto más lejos, es poniéndome sobre los hombros de Gigantes*

Un registro de pasos / tutorial basado en el blog de [Stephen Marz](https://osblog.stephenmarz.com) de cómo crear un sistema operativo para la plataforma RISC-V utilizando el lenguaje Rust.

Este OS no pretende ser un *copy paste* del ya implementado por Stephen (Que por cierto es muy groso y debería leerlo y aportar en su Patreon y etc), sino abordarlo desde un enfoque distinto, con pasos más chicos y un intento de TDD.

¿Por qué RISC-V? Porque es una arquitectura abierta, libre, lo que favorece mucho a su investigación. Sus instrucciones son sencillas y fáciles de decodificar, además es modular, por lo que podemos agregar módulos de instrucciones según vayamos necesitando.

¿Por qué Rust? Porque Rust es un lenguaje moderno, que le pone énfasis a la seguridad de memoria, concurrencia, etc. Tiene un compilador muy, muy bueno, y una muy buena comunidad.


## Armando el entorno

Lo primero que tenemos que hacer es descargar un toolchain para RISC-V.
Por suerte Rust tiene incorporado el toolchain suyo de forma oficial, lo que nos facilita mucha configuración.
También es necesario instalar QEmu. En ubuntu se puede instalar el paquete `qemu-system-misc` con el comando

~~~{.bash}
sudo apt install qemu-system-misc
~~~

El mismo trae `qemu-system-riscv32` y `qemu-system-riscv64`

Debido a que la versión del repositorio de ubuntu puede ser un poco vieja (y en algunas versiones presenta bugs importantes) recomiendo clonar desde el repo oficial:	

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


## Dispositivo UART
Creamos un dispositivo *UART*. Para esto, sabiendo que QEMU designa la posición de memoria *0x1000_0000* para virtualizar un dispositivo NS16550 "Instanciamos" uno en nuestro kmain para informar sobre la ejecución del kernel.

Más información del mapeo de memoria: https://www.sifive.com/blog/risc-v-qemu-part-1-privileged-isa-hifive1-virtio
Un dispositivo UART nos va a ayudar mucho a debuggear nuestro sistema, ya que nos permite comunicarnos por medio del puerto serie.

En nuestras pruebas en python vamos a agregarle la opción `stdout=PIPE` a nuestro proceso de QEmu para poder leer la salida del kernel y compararla contra nuestras pruebas


## Tests en Rust

Rust tiene un framework de test incorporado al sistema de compilación, sin embargo este necesita de la biblioteca `std` para poder ejecutarse. Por suerte podemos utilizar nuestro propio "framework" de pruebas. Para esto definimos una función como `test_runner`, la cual compilamos únicamente si estamos corriendo tests. 

Al declarar nuestro *runner*, se creará automáticamente una función "`main`" que llamará a nuestro wrapper. Podemos renombrar esta función configurando el `reexport_test_harness_main`

Una vez configurados los wrappers de nuestras pruebas, hacemos una llamada al *harness* desde nuestro `kmain`.

Por el momento esto nos puede ayudar a ejecutar tests sencillos, aunque deberá ser remodelado cuando se separe el modo máquina del modo supervisor / usuario.


## Paginas de memoria

Exportamos direcciones de memoria con el script del linkeditor y luego usamos un snippet en assembly para guardar esas direcciones en variables globales. Luego definiremos un tamaño de página de memoria standard, de 4kb, y luego, conociendo la dirección del heap y su tamaño, procedemos a dividir el heap en páginas.

Utilizamos el dispositivo UART para imprimir las direcciones de memoria que identificamos.


### Alloc

En este paso hacemos un alloc primitivo de páginas. Recibo por parámetro la cantidad de páginas que quiero reservar de forma continua.

Iteramos la lista de páginas y buscamos una secuencia de N páginas libres. Si la encontramos, devolvemos el puntero.


### Dealloc

Implementamos un dealloc para liberar la memoria reservada por `alloc`. Hacemos un chequeo básico de no estar liberando un puntero nulo o un double free.


## MMU (sin page faults)

RISC-V es una arquitectura modular, algunas de sus implementaciones son muy similares a las de un microcontrolador: muy pocas instrucciones, sin instrucciones de privilegio y sin MMU. Por este motivo, el MMU es un hardware externo al procesador.
Cuando el sistema corre en el anillo de seguridad 1 (modo *M*), el de máquina, la *MMU* está desactivada y hacemos uso directo de la memoria.
Cuando pasamos al anillo 2, el de supervisor, activamos la *MMU*.

La *MMU* se configura con el registro **SATP** (Supervisor Address Translation and Protection). Este registro tiene 3 campos:

* **MODE**: que define el tipo de transformación (0 si se usa memoria física, otros valores según si es RISCV-32 o RISCV-64).

* **ASID**: utilizado para asociar un espacio de memoria (*address space*) a un proceso. Podemos elegir 0 y recargar toda la TLB, o usar algo único como el PID para solo recargar si es necesario.

* **PPN**: *Physical page number*, la *dirección* de la página donde va a estar alojada nuestra TLB. Se le quitan los 12 primeros bits (porque las páginas son de 2^12 bytes).

En esta versión no atendemos las interrupciones por Page Faults y todavía no habilitamos el modo supervisor.

### Map

Implementamos la función `map`, con la que asignamos una dirección de memoria física a una dirección virtual.
Usaremos el modelo de paginado `SV39`. Esto significa que dividimos la dirección virtual en 3 partes de 9 bits: VPN2, VPN1 y VPN0 (Virtual Page Number), cada una de estas partes consta de 9 bits y refieren al índice de una página de memoria virtual. Existen otras configuraciones, como SV48 de 4 niveles de 8 bits, o SV32, similar a SV39 pero para arquitecturas de 32bits
De esta forma RISC-V nos ofrece 3 niveles de paginación: paginas de 4KB para el nivel 0, de 2MB para nivel 1 o de 1GB para nivel 2
Empezamos con una página `root`, que contiene un índice de entradas de páginas virtuales. Si el índice posee alguno de los flags `RWX` (Lectura, Escritura o Ejecución), entonces es esta página es una hoja.

Nuestra función map recibe la dirección de la tabla raiz, la dirección virtual, la física a la que queremos mapear, y el nivel de la página.


## Modo supervisor

El módo *Máquina* utiliza memoria física, si queremos usar memoria virtual necesitamos pasar al modo *Supervisor* o *Usuario*. Para esto convertimos nuestra función *kmain* en *kinit*, donde se inicializan los dispositivos, paginación de memoria y memoria virtual. Una vez inicializado devolvemos el valor del registro `satp`. Este registro posee la ubicación (alineada a una página de memoria) de la raiz de nuestro **TLB**, como se describió anteriormente. Se configura este registro y se hace un retorno de interrupción para pasar al modo supervisor.

En este modo se utiliza memoria virtual, como se configuró la MMU. Sin embargo, como ciertas configuraciones las hicimos en modo máquina, tenemos que configurar la tabla de mapeo para poder acceder a la memoria con las instrucciones, dispositivos, etc. De lo contrario sucederá un *Page Fault*, y al no tener interrupciones configuradas, nuestro kernel quedará loopeando infinitamente.


## Interrupciones

Esta parte fue muy complicada de sacar andando. En el tutorial retrocede a modo máquina para atender las interrupciones, sin embargo, en modo supervisor es donde anda la memoria virtual, así que... ¡A hacerlo andar en modo supervisor!

Lo primero que hacemos es completar nuestra función `asm_trap_vector`, en la que ahora hacemos una copia de todos los registros y el registro de control y status (CSR) `mscratch`. Una vez que copiamos al stack todos nuestros registros, saltamos a la función `m_trap_handler` para analizar el motivo de la interrupción. Por el momento sólo adelantamos el *program counter* o ejecutamos un *panic!* según el motivo. Una vez que imprimimos el motivo de nuestra interrupción, volvemos a la ejecución

**IMPORTANTE**: hay que tener en cuenta que, si estamos sobre una instrucción comprimida, al sumar 4 al *program counter* podemos caer sobre una instrucción no alineada. Esto rompería con la ejecución de nuestro sistema. Sin embargo, más adelante veremos que, en vez de adelantar el *program counter*, podemos llamar a funciones más específicas para este tipo de situaciones.
