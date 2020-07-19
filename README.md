# BarbaOS

Sistema operativo hecho en Rust para RiscV

# Tests

Testing en un sistema operativo es extraño, pero no imposible.

Para ejecutar los tests necesitamos

* *pytest*: framework en el que están escritos los tests

* *pygdbmi*: biblioteca para comunicarse con gdb

Instalarlos con los comandos

~~~{.bash}
pip3 install pytest pygdbmi
~~~

## Ejecución

Ejecutar los test con

```bash
pytest
```

Se recomienda usar `flake8` para verificar reglas de linter.

# Debugger

* Ejecutar SO en una terminal: `cargo run -- -s -S`

* Ejecutar `riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/debug/barbaos`, y conectar al proceso lanzado con el comando `target remote :1234`.
