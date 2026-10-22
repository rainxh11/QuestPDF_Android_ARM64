"""Verify ELF ABI, Android dependencies, 16 KB segments and managed P/Invoke exports."""
import argparse
from pathlib import Path
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(__doc__)
parser.add_argument('directory', type=Path)
parser.add_argument('--abi', choices=['arm64-v8a', 'x86_64'], required=True)
parser.add_argument('--llvm-bin', type=Path, required=True)
args = parser.parse_args()


def tool(name, *params):
    executable = args.llvm_bin / name
    if not executable.exists():
        executable = executable.with_suffix('.exe')
    return subprocess.check_output([str(executable), *map(str, params)], text=True)


for name in ['questpdf_skia', 'questpdf_qpdf', 'questpdf_android']:
    path = args.directory / f'lib{name}.so'
    data = path.read_bytes()
    assert data[:6] == b'\x7fELF\x02\x01', f'{path}: expected little-endian ELF64'
    machine = struct.unpack_from('<H', data, 18)[0]
    assert machine == (183 if args.abi == 'arm64-v8a' else 62), f'{path}: wrong machine {machine}'
    phoff = struct.unpack_from('<Q', data, 32)[0]
    phentsize, phnum = struct.unpack_from('<HH', data, 54)
    loads = 0
    for index in range(phnum):
        kind, flags, offset, virtual, _, filesz, memsz, align = struct.unpack_from('<IIQQQQQQ', data, phoff + index * phentsize)
        if kind != 1:
            continue
        loads += 1
        assert align >= 16384 and (virtual - offset) % 16384 == 0, f'{path}: not 16 KB aligned'
        assert not (flags & 1 and flags & 2), f'{path}: writable executable segment'
    assert loads, f'{path}: missing LOAD segments'
    dynamic = tool('llvm-readelf', '-d', path)
    dependencies = re.findall(r'\(NEEDED\).*?\[(.*?)\]', dynamic)
    allowed = {'libandroid.so', 'liblog.so', 'libz.so', 'libm.so', 'libdl.so', 'libc.so'}
    assert set(dependencies) <= allowed, f'{path}: unexpected dependencies: {dependencies}'
    versions = tool('llvm-readelf', '--version-info', path)
    assert 'GLIBC_' not in versions, f'{path}: glibc build is not an Android library'
    symbols = {line.split()[-1].split('@')[0] for line in tool('llvm-nm', '--dynamic', '--defined-only', path).splitlines() if line.strip()}
    if name == 'questpdf_android':
        required = {'questpdf_abi_version', 'questpdf_generate', 'questpdf_run_tests', 'questpdf_free'}
    else:
        folder = 'Skia' if name == 'questpdf_skia' else 'Qpdf'
        source = '\n'.join(p.read_text(encoding='utf-8-sig') for p in (ROOT / 'src/dotnet/library/QuestPDF' / folder).rglob('*.cs'))
        required = set(re.findall(r'extern\s+[\w<>\[\]?]+\s+((?:questpdf_|qpdf)[a-zA-Z0-9_]+)\s*\(', source))
    assert required <= symbols, f'{path}: missing imports {sorted(required - symbols)}'
    print(f'{args.abi} {path.name}: {len(data):,} bytes; {loads} aligned LOAD segments; {len(required)} required exports; dependencies {dependencies}')
