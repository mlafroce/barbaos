from pygdbmi.gdbcontroller import GdbController
from subprocess import Popen, PIPE

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


@pytest.fixture
def qemu_process():
    """
    Lanza una instancia de qemu en un proceso aparte
    """
    process = Popen(QEMU_COMMAND, stdout=PIPE)
    return process


@pytest.fixture
def gdbmi():
    """
    Inicializa el controlador de gdb
    """
    controller = GdbController([DEBUGGER, KERNEL, "--interpreter=mi3"])
    controller.write("target remote :1234")
    return controller


def test_hello(qemu_process, gdbmi):
    """
    Test impresion hello world
    """
    gdbmi.write("c")
    output = qemu_process.stdout.readline().strip()
    assert output == b"Hello Rust!"


if __name__ == "__main__":
    print("run `$ pytest`")
