from pygdbmi.gdbcontroller import GdbController
from subprocess import Popen, PIPE, run

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


def pytest_configure():
    run(["cargo", "build"])


@pytest.fixture
def qemu_process():
    """
    Lanza una instancia de qemu en un proceso aparte
    """
    process = Popen(QEMU_COMMAND, stdout=PIPE)
    yield process
    process.kill()


@pytest.fixture
def gdbmi():
    """
    Inicializa el controlador de gdb
    """
    controller = GdbController([DEBUGGER, KERNEL, "--interpreter=mi3"])
    controller.write("target remote :1234")
    return controller
