"""Check native output encoders independently of PDF raster comparisons."""
from pathlib import Path
import argparse
from PIL import Image
import numpy as np
p=argparse.ArgumentParser();p.add_argument('reference');p.add_argument('actual');a=p.parse_args();r,v=Path(a.reference),Path(a.actual)
for name in ['opaque.png','transparent.png','jpeg.jpg','webp.webp']:
    x=np.array(Image.open(r/name).convert('RGBA'));y=np.array(Image.open(v/name).convert('RGBA'))
    assert x.shape==y.shape and np.array_equal(x,y),name+' decoded pixels differ'
    assert (r/name).read_bytes()==(v/name).read_bytes(),name+' encoded bytes differ'
    print(name,'PASS encoded bytes and pixels')
assert(r/'page.svg').read_bytes()==(v/'page.svg').read_bytes()
assert(r/'decimal.txt').read_text().splitlines()==['9,007,199,254,740,993.13','-2.35','0.30']
print('SVG PASS identical XML bytes; C# decimal reference PASS')
import json
for name in ['fonts-registered.json','fonts-system.json']:
    def fonts(path):return sorted(tuple(sorted((key.replace('_','').lower(),value)for key,value in row.items()))for row in json.loads(path.read_text()))
    assert fonts(r/name)==fonts(v/name),name+' font enumeration differs'
    print(name,'PASS',len(fonts(v/name)),'typefaces')
