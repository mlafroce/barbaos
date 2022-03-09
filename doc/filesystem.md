# Filesystem

Un filesystem nos permite almacenar y cargar datos desde un dispositivo de almacenamiento (como un disco rígido). Existen varios tipos de filesystems, el más utilizado en linux es **Ext4**, que es una extensión a los sistemas Ext3 y **Ext2**. A su vez, Ext2 está basado en el filesystem **MinixFS**.

Otros filesystems populares son los de tipo **FAT**, que utilizan una lista de alocaciones, y **NTFS**, utilizado por Windows posteriores a NT. **USTAR** si bien no es un filesystem en sí, es el formato de almacenamiento utilizado por linux para los archivos `.tar`

Elegimos **Ext2**, que si bien es complejo, se lo puede utilizar de forma nativa con Linux.

## Ext 2

### General

Ext2 divide el disco en bloques lógicos de tamaño fijo. El primero de estos bloques se llama **superblock**, y posee información importante del sistema de archivos, como tamaño de bloques, cantidad, cantidad de inodos, etc.

La información almacenada en los bloques es indexada por **Inodos**. Cada archivo o directorio es descripto por un Inodo, que posee información como *owner*, permisos, tamaño del archivo, y, lo más importante, una lista de bloques correspondientes a los datos del archivo. Esta lista posee punteros de 3 tipos:

* Indices directos a bloques, es decir, indices de bloques que contienen la información de mi archivo.
* Indices de profundidad 2, que apuntan a un bloque que contiene más punteros a bloques.
* Indices de profundidad 3, que apuntan a un bloque que apunta a otro bloque de punteros que finalmente apuntan a los datos de mi archivo.

Para reducir la fragmentación, los bloques se agrupan en **block-groups**. Cada grupo tiene una copia del *superbloque* (en la primera revisión de Ext2, en la segunda solo algunos grupos). Inmediatamente después del superbloque está la lista de *group descriptors*. Esta es una lista de una estructura que posee:

* id del bitmap de bloques utilizados
* id del bitmap de inodos utilizados
* tabla de inodos del grupo de bloques

La cantidad de block groups se calcula mediante la *cantidad total de bloques / bloques por grupo* en el *superblock*. Multiplicando la cantidad de block groups por el tamaño en bytes de los descriptores puedo estimar la cantidad de bloques utilizados para esta tabla de group descriptors.

### Navegación

El directorio raiz es siempre el inodo 2. Si leemos esta entrada en la tabla de inodos, podemos comprobar que es un directorio, y podemos navegar su contenido.

Los bloques de datos de los directorios poseen una lista de entradas de directorio, que son estructuras que indican principalmente el inodo, tipo de archivo y nombre del archivo (o directorio) dentro del directorio.

Es decir, si nosotros queremos ingresar a `/home/matias`, debemos inspeccionar el inodo 2, recorrer los bloques en la lista de `i_blocks`, parsear los nombres de archivo hasta llegar a `home`, acceder al inodo asociado, y repetir la inspección pero buscando el archivo `matias`.


## Armar un disco de prueba y montarlo

Para crear un archivo de disco primero creamos un archivo grande en blanco:

```
dd if=/dev/zero of=hdd.img  bs=1M  count=16
```

Para montarlo usamos un *loop device*:

```
sudo losetup --partscan --find --show hdd.img
```

Esto nos va a dar una salida del estilo

```
/dev/loop42 
```

Una vez que está montado podemos darle formato con herramientas como `fdisk` o `parted`. Por ejemplo:

```
sudo fdisk /dev/loop42
```

Elegimos el comando `o` para crear una tabla de particiones clásica MBR, y `n` para crear una partición.
Salimos guardando los cambios con `w`.

Podremos observar que se creó un dispositivo loop nuevo para nuestra partición, con el mismo nombre que la anterior + "p1" 

Una vez formateado la nueva partición puede ser montada en una carpeta vacía para su edición.
Por ejemplo, suponiendo que tenemos la carpeta vacía `my-disk`, podemos ejecutar:

```
sudo mount /dev/loop42p1 ./my-disk 
```

Una vez finalizada la edición, desmontamos y cerramos el dispositivo:

```
sudo umount /dev/loop42p1
sudo losetup --detach /dev/loop42
```

Para probar nuestro SO, agregaremos una carpeta `boot` con un archivo `boot.md`.
