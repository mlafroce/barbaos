from conftest import qemu_process, gdbmi
from time import sleep

import pytest


def test_boot_info(qemu_process, gdbmi):
    """
    Test salida información booteo
    """
    gdbmi.write("c")
    output = qemu_process.stdout.readlines()
    stripped = list(map(lambda line: line.strip(), output))
    stripped.index(b"BarbaOS booting...")
    stripped.index(b"Compatible: riscv-virtio")
    #stripped.index(b"MMU         : riscv,sv48")
    stripped.index(b"Memory start: 0x80000000")


if __name__ == "__main__":
    print("run `$ pytest`")
