"""Check every Android/Bionic native asset and managed target in a local NuGet package."""
import argparse
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(__doc__)
parser.add_argument('package', type=Path)
args = parser.parse_args()

frameworks = ['net10.0', 'net8.0', 'net6.0', 'net5.0', 'netstandard2.0']
rids = [f'{os}-{cpu}' for os in ['android', 'linux-bionic'] for cpu in ['arm', 'arm64', 'x64']]
with zipfile.ZipFile(args.package) as package:
    for framework in frameworks:
        assert package.read(f'lib/{framework}/QuestPDF.dll')[:2] == b'MZ', f'Missing {framework} assembly'
    for rid in rids:
        for library in ['libquestpdf_skia.so', 'libquestpdf_qpdf.so']:
            entry = f'runtimes/{rid}/native/{library}'
            expected = ROOT / 'src/dotnet/library/QuestPDF/Runtimes' / rid / 'native' / library
            assert package.read(entry) == expected.read_bytes(), f'Package asset differs from source: {entry}'
print(f'{args.package.name}: {len(frameworks)} managed frameworks and {len(rids)} Android/Bionic RIDs; all 12 native assets match source')
