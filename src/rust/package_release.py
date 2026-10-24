"""Bundle Rust executables and their native dependencies, preserving .NET."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, help='New, nonexistent destination directory')
p.add_argument('--rid', choices=['win-x64','linux-x64'], default='win-x64')
p.add_argument('--binaries', type=Path, default=ROOT/'target/release')
p.add_argument('--skip-build', action='store_true', help='Package an already built target, e.g. Linux Docker output')
a = p.parse_args()
destination = (a.output or ROOT / 'output' / ('release-' + a.rid + '-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))).resolve()
if destination.exists():
    raise SystemExit(f'Refusing to replace an existing directory: {destination}')
if not a.skip_build:
    subprocess.run(['cargo', 'build', '--workspace', '--release', '--locked'], cwd=ROOT, check=True)
destination.mkdir(parents=True)
files = ['receipt-preview.exe','ris_document_reports.dll'] if a.rid=='win-x64' else ['receipt-preview','libris_document_reports.so']
for name in files:
    shutil.copy2(a.binaries / name, destination / name)
native = ROOT / ('../dotnet/library/QuestPDF/Runtimes/'+a.rid+'/native')
libraries = ['questpdf_skia.dll','questpdf_qpdf.dll'] if a.rid=='win-x64' else ['libquestpdf_skia.so','libquestpdf_qpdf.so']
for name in libraries:
    shutil.copy2(native / name, destination / name)
for source, name in [('document-server/assets', 'Assets'), ('verification/fixtures', 'fixtures')]:
    shutil.copytree(ROOT / source, destination / name)
for source in ['LICENSE.md', 'LICENSE-APACHE-2.0.txt', 'THIRD_PARTY_NOTICES.md', 'README.md', 'PORT_STATUS.md', 'VERIFICATION.md']:
    shutil.copy2(ROOT / source, destination / source)
shutil.copy2(ROOT / 'questpdf/resources/LatoFont/OFL.txt', destination / 'Lato-OFL.txt')
shutil.copy2(ROOT / 'document-server/include/ris_document_reports.h', destination)
manifest = dict(source_version='2026.9.1', skia_abi=26, qpdf_abi=4, platform=a.rid, files={})
for path in sorted(destination.rglob('*')):
    if path.is_file():
        manifest['files'][path.relative_to(destination).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
(destination / 'manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
archive = destination.with_suffix('.zip')
if archive.exists():
    raise SystemExit(f'Refusing to overwrite archive: {archive}')
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as output:
    for path in sorted(destination.rglob('*')):
        if path.is_file():
            info=zipfile.ZipInfo.from_file(path,path.relative_to(destination))
            if a.rid.startswith('linux'):
                info.create_system=3
                info.external_attr=(0o100755 if path.name in files else 0o100644)<<16
            output.writestr(info,path.read_bytes(),compress_type=zipfile.ZIP_DEFLATED)
print(destination)
print(archive)
