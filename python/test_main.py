from config import qemu_process, gdbmi
from time import sleep

import pytest

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
