"""Inspect actual qpdf operation results with an independent PDF reader."""
from pathlib import Path
import argparse,pymupdf as pdf
p=argparse.ArgumentParser();p.add_argument('output',nargs='?',default='output/operations');a=p.parse_args();root=Path(a.output)
def texts(name):
    with pdf.open(root/name)as d:return[d[i].get_text()for i in range(len(d))]
assert len(texts('base.pdf'))==3
assert ['Source page 3' in texts('selected.pdf')[0],'Source page 1' in texts('selected.pdf')[1]]==[True,True]
assert len(texts('selected.pdf'))==2
assert len(texts('merged.pdf'))==4 and 'OVERLAY' in texts('merged.pdf')[3]
assert b'/Linearized'in(root/'merged.pdf').read_bytes()[:1024]
for name in ['overlay.pdf','underlay.pdf']:
    assert len(texts(name))==3 and all('OVERLAY'in t for t in texts(name))
with pdf.open(root/'encrypted.pdf')as d:
    assert d.is_encrypted
    assert not d.authenticate('incorrect')
    assert d.authenticate('fixture-read')
    assert not d.permissions & pdf.PDF_PERM_PRINT
    assert 'Source page 1'in d[0].get_text()
with pdf.open(root/'decrypted.pdf')as d:assert not d.is_encrypted and len(d)==3
assert texts('decrypted.pdf')==texts('base.pdf')
for bits in [40,128,256]:
    with pdf.open(root/f'encrypted-{bits}.pdf')as d:
        assert d.is_encrypted and not d.authenticate('incorrect') and d.authenticate('fixture-read')
        for permission in [pdf.PDF_PERM_PRINT,pdf.PDF_PERM_COPY,pdf.PDF_PERM_ANNOTATE,pdf.PDF_PERM_MODIFY]:assert not(d.permissions & permission),(bits,permission)
        if bits>=128:
            for permission in [pdf.PDF_PERM_ASSEMBLE,pdf.PDF_PERM_FORM]:assert not(d.permissions & permission),(bits,permission)
        assert f'{bits}-bit'in d.metadata['encryption'],d.metadata['encryption']
with pdf.open(root/'attached.pdf')as d:
    assert d.embfile_count()==1
    assert d.embfile_get(0)==b'Rust attachment fixture'
    info=d.embfile_info(0);assert info['filename']=='note.txt'
with pdf.open(root/'archival.pdf')as d:
    assert '<pdfaid:part>3</pdfaid:part>'in d.get_xml_metadata()
    assert '<pdfaid:conformance>B</pdfaid:conformance>'in d.get_xml_metadata()
with pdf.open(root/'extended.pdf')as d:
    from xml.etree import ElementTree as ET
    metadata=ET.fromstring(d.get_xml_metadata())
    assert metadata.find('.//{urn:questpdf-rust:test}value').text=='Verified XMP extension'
print('PDF operations: selection/order, merge, linearization, layers, encryption/permissions, decryption, attachment contents and XMP extension passed')
