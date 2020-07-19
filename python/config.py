from pygdbmi.gdbcontroller import GdbController
from subprocess import Popen, PIPE

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
