from pygdbmi import gdbmiparser
from conftest import qemu_process
from time import sleep

import pytest
import re

def call_dealloc(gdbmi, ptr):
    gdbmi.write(f"call page_table.dealloc({ptr})")

def set_alloc(gdbmi, name, size):
    gdbmi.write(f"set ${name} = page_table.alloc({size})")

def get_alloc(gdbmi, name):
    alloc = gdbmi.write(f"p ${name}.0")
    try:
        return find_and_parse_hex(alloc[1]["payload"])
    except:
        return 0

def find_and_parse_hex(string):
    # Find all occurrences of "0x" followed by hexadecimal digits
    match = re.search(r'0x[0-9a-fA-F]+', string)
    return int(match.group(0), 16)

def test_alloc(qemu_process, gdbmi):
    """
    Test de multiples allocs
    """
    gdbmi.write("b barbaos::mmu::riscv64::PageTable::init")
    gdbmi.write("c")
    gdbmi.write("fin")

    set_alloc(gdbmi, "first", 10000)
    set_alloc(gdbmi, "second", 20000)
    set_alloc(gdbmi, "fail", 10000)

    assert(get_alloc(gdbmi, "fail") == 0)
    call_dealloc(gdbmi, get_alloc(gdbmi, "second"))
    set_alloc(gdbmi, "some10k", 10000)
    set_alloc(gdbmi, "other10k", 10000)

    assert(get_alloc(gdbmi, "other10k") != 0)

    gdbmi.write("c")

    output = qemu_process.stdout.readlines()
