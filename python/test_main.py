from pygdbmi.gdbcontroller import GdbController
from subprocess import Popen, PIPE
from time import sleep

import pytest

DEBUGGER = "riscv64-unknown-linux-gnu-gdb"
KERNEL = "../target/riscv64gc-unknown-none-elf/debug/barbaos"

QEMU_COMMAND = ["qemu-system-riscv64",
                "-machine", "virt",
                "-nographic",
                "-serial", "mon:stdio",
                "-bios", "none",
                "-smp", "4",
                "-m", "128M",
                "-kernel", KERNEL,
                "-s", "-S"]

"""
Lanza una instancia de qemu en un proceso aparte
"""
@pytest.fixture
def qemu_process():
    process = Popen(QEMU_COMMAND, stdout=PIPE)
    yield process
    process.kill()

"""
Inicializa el controlador de gdb
"""
@pytest.fixture
def gdbmi(): 
    controller = GdbController([DEBUGGER, KERNEL, "--interpreter=mi3"])
    controller.write("target remote :1234")
    return controller
    
"""
Test básico de levantar una instancia y matarla
"""
def test_kill(qemu_process, gdbmi):
    gdbmi.write("k")

def test_should_timeout(qemu_process, gdbmi):
    response = gdbmi.write("c")
    with pytest.raises(Exception):
        response = gdbmi.write("c")

"""
Test impresión hello world
"""
def test_hello(qemu_process, gdbmi):
    gdbmi.write("c")
    sleep(0.01)
    output = qemu_process.stdout.readline().strip()
    assert output == b"Hello Rust!"
