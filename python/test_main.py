from pygdbmi.gdbcontroller import GdbController
from subprocess import Popen

import pytest

DEBUGGER = "gdb-multiarch"
KERNEL = "target/riscv64gc-unknown-none-elf/debug/barbaos"

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
    process = Popen(QEMU_COMMAND)
    yield process
    process.kill()

"""
El 
"""
def test_kill(qemu_process):
    gdbmi = GdbController([DEBUGGER, KERNEL, "--interpreter=mi3"])

    gdbmi.write("target remote :1234")
    gdbmi.write("k")

def test_should_timeout(qemu_process):
    gdbmi = GdbController([DEBUGGER, KERNEL, "--interpreter=mi3"])

    gdbmi.write("target remote :1234")
    response = gdbmi.write("c")
    with pytest.raises(Exception):
        response = gdbmi.write("c")
