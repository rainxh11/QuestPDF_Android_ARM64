"""Compare native image generation and ESC/POS output without sending print jobs."""
from pathlib import Path
import argparse,json
import numpy as np
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('reference');p.add_argument('actual');p.add_argument('--output',default='output/image-comparison.json');a=p.parse_args()
reference,actual=Path(a.reference),Path(a.actual);results=[]
for source in sorted(reference.glob('*.png')):
    target=actual/source.name
    if not target.exists():results.append(dict(name=source.name,equal=False,missing=True));continue
    x=np.array(Image.open(source).convert('RGBA'));y=np.array(Image.open(target).convert('RGBA'))
    same=x.shape==y.shape
    result=dict(name=source.name,reference_shape=x.shape,actual_shape=y.shape,equal=same and np.array_equal(x,y))
    if same:result['changed_pixels']=int(np.any(x!=y,axis=2).sum())
    results.append(result)
for source in sorted(reference.glob('*escpos.bin')):
    target=actual/source.name
    x=source.read_bytes();y=target.read_bytes()if target.exists()else b''
    results.append(dict(name=source.name,equal=x==y,reference_length=len(x),actual_length=len(y),changed_bytes=sum(i!=j for i,j in zip(x,y))+abs(len(x)-len(y))))
for r in results:print(r['name'],'PASS'if r['equal']else'FAIL',r.get('changed_pixels',r.get('changed_bytes')))
Path(a.output).write_text(json.dumps(results,indent=2),encoding='utf-8')
raise SystemExit(0 if results and all(r['equal']for r in results)else 1)
