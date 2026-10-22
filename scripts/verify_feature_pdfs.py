"""Independent structural and rendering checks of Android-produced feature PDFs.
Requires pypdf and Pillow; Poppler is supplied with --poppler-bin.
"""
import argparse
import json
from pathlib import Path
import subprocess
from pypdf import PdfReader
from PIL import Image

parser = argparse.ArgumentParser(__doc__)
parser.add_argument('directory', type=Path)
parser.add_argument('--poppler-bin', type=Path, required=True)
args = parser.parse_args()
directory = args.directory
report = json.loads((directory / 'results.json').read_text())
assert all(result['Passed'] for result in report['Results']), 'Native suite has failures'
renders = directory / 'renders'
renders.mkdir(exist_ok=True)
documents = {}
page_count = 0

for result in report['Results']:
    id = result['Id']
    path = directory / f'feature-{id:02}.pdf'
    reader = PdfReader(path, strict=True)
    documents[id] = reader
    assert len(reader.pages) > 0
    if id == 0:
        assert reader.metadata.title == result['Name']
    if id == 1:
        assert len(reader.pages) >= 4
        text = '\n'.join(page.extract_text() for page in reader.pages)
        assert 'item 001' in text and 'item 160' in text
        assert all('Description' in page.extract_text() for page in reader.pages)
    if id in (6, 10):
        assert len(reader.pages) == 2
    if id == 6:
        assert len(reader.pages[0]['/Annots']) >= 2
    if id == 8:
        xmp = reader.xmp_metadata
        assert xmp.pdfaid_part == '3' or xmp.pdfaid_part == 3
        assert xmp.pdfaid_conformance == 'B'
        assert '/OutputIntents' in reader.trailer['/Root']
    if id == 9:
        root = reader.trailer['/Root']
        assert '/StructTreeRoot' in root and '/MarkInfo' in root and '/Lang' in root
    if id == 10:
        assert 'Second page' in reader.pages[0].extract_text()
        assert 'WATERMARK' in reader.pages[1].extract_text()
    if id == 11:
        assert '/XObject' in reader.pages[0]['/Resources']
    if id == 12:
        attachments = reader.attachments
        assert 'pièce-é.txt' in attachments
        assert 'مرحبا' in attachments['pièce-é.txt'][0].decode('utf-8')
    if id == 20:
        metadata = reader.trailer['/Root']['/Metadata'].get_object().get_data()
        assert b'urn:questpdf:android' in metadata and b'ARM64' in metadata
    for page in reader.pages:
        assert float(page.mediabox.width) > 0
        assert page.extract_text().strip()
        resources = page.get('/Resources')
        if resources and '/Font' in resources:
            for ref in resources['/Font'].values():
                font = ref.get_object()
                if font.get('/Subtype') == '/Type0':
                    desc = font['/DescendantFonts'][0].get_object()['/FontDescriptor'].get_object()
                    assert '/FontFile2' in desc or '/FontFile3' in desc
    executable = args.poppler_bin / 'pdftoppm.exe'
    if not executable.exists(): executable = args.poppler_bin / 'pdftoppm'
    subprocess.run([str(executable), '-scale-to', '650', '-png', str(path), str(renders / f'feature-{id:02}')], check=True, capture_output=True)
    page_count += len(reader.pages)

for id in (13, 14):
    reader = PdfReader(directory / f'encrypted-{id}.pdf')
    assert reader.is_encrypted
    assert reader.decrypt('wrong-password') == 0
    # PDF revision 6 passwords are UTF-8, whereas revision 4 uses PDFDocEncoding.
    # Pass bytes explicitly: pypdf's string path first tries Latin-1 for U+00E9.
    assert reader.decrypt('user-é'.encode('utf-8') if id == 14 else 'user-é') != 0
    assert len(reader.pages) == 2
for name, format in [('Png', 'PNG'), ('Jpeg', 'JPEG'), ('Webp', 'WEBP')]:
    with Image.open(directory / f'{name}.image') as image:
        assert image.format == format and image.width > 500
assert '<svg' in (directory / 'export.svg').read_text()
print(f"{report['Runtime']}: {len(documents)} PDFs, {page_count} pages parsed and rendered; metadata, fonts, links, attachments, encryption and exports verified")
