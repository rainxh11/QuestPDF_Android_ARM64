"""Compare pages, extracted text/positions and raster output, never PDF file hashes.

Exit nonzero for any difference. JSON preserves quantitative evidence per page.
"""
from pathlib import Path
import argparse,json
import pymupdf as fitz
import numpy as np
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('reference');p.add_argument('actual');p.add_argument('--output',default='output/comparison');p.add_argument('--dpi',type=int,default=144);p.add_argument('--cases',help='Engine manifest: require every success/error expectation');a=p.parse_args()
out=Path(a.output);out.mkdir(parents=True,exist_ok=True)
results=[]
reference_dir=Path(a.reference);actual_dir=Path(a.actual)
names=sorted({p.stem for directory in (reference_dir,actual_dir) for p in directory.iterdir() if p.suffix in ('.pdf','.error')})
expectations={c['name']:c.get('expected_error',False)for c in json.loads(Path(a.cases).read_text())}if a.cases else None
if expectations is not None:names=sorted(set(names)|set(expectations))
for name in names:
    actual=actual_dir/(name+'.pdf');ref=reference_dir/(name+'.pdf')
    re=reference_dir/(name+'.error');ae=actual_dir/(name+'.error')
    if re.exists()or ae.exists():
        results.append({'name':name,'equal':re.exists()and ae.exists()and not ref.exists()and not actual.exists()and(expectations is None or expectations.get(name,False)),'reference_pages':0,'rust_pages':0,'pages':[],'expected_error':re.read_text(encoding='utf-8')if re.exists()else None,'actual_error':ae.read_text(encoding='utf-8')if ae.exists()else None})
        continue
    if not ref.exists()or not actual.exists():
        results.append({'name':name,'equal':False,'reference_pages':0,'rust_pages':0,'pages':[],'missing_reference':not ref.exists(),'missing_actual':not actual.exists()})
        continue
    r=fitz.open(ref);v=fitz.open(actual)
    result={'name':actual.stem,'reference_pages':len(r),'rust_pages':len(v),'pages':[],'equal':len(r)==len(v)and(expectations is None or name in expectations and not expectations[name])}
    for i in range(min(len(r),len(v))):
        x,y=r[i],v[i];px=x.get_pixmap(dpi=a.dpi,alpha=False);py=y.get_pixmap(dpi=a.dpi,alpha=False)
        xa=np.frombuffer(px.samples,dtype=np.uint8).reshape(px.height,px.width,3);ya=np.frombuffer(py.samples,dtype=np.uint8).reshape(py.height,py.width,3)
        text_equal=x.get_text()==y.get_text();size_equal=all(abs(u-v)<.01 for u,v in zip(x.rect,y.rect))
        def spans(page):return [(s['text'],s['size'],s['color'],s['bbox'])for b in page.get_text('dict')['blocks']if b['type']==0 for l in b['lines']for s in l['spans']]
        xs,ys=spans(x),spans(y)
        positions_equal=len(xs)==len(ys)and all(u[:3]==v[:3]and max(abs(i-j)for i,j in zip(u[3],v[3]))<=.001 for u,v in zip(xs,ys))
        def links(page):
            result=[]
            for link in page.get_links():
                result.append((link.get('kind'),link.get('uri'),link.get('page'),[round(x,3)for x in link['from']],tuple(round(x,3)for x in link.get('to',()))))
            return result
        links_equal=links(x)==links(y)
        same_shape=xa.shape==ya.shape
        difference=np.abs(xa.astype(np.int16)-ya.astype(np.int16)) if same_shape else None
        equal=same_shape and np.array_equal(xa,ya)
        page={'page':i+1,'reference_size':list(x.rect),'rust_size':list(y.rect),'text_equal':text_equal,'text_positions_equal':positions_equal,'links_equal':links_equal,'size_equal':size_equal,'pixels_equal':equal}
        if difference is not None:page.update(changed_pixels=int(np.any(difference>0,axis=2).sum()),maximum_channel_difference=int(difference.max()),mean_channel_difference=float(difference.mean()))
        result['pages'].append(page);result['equal']&=equal and text_equal and positions_equal and size_equal and links_equal
        if not equal or not positions_equal or not links_equal:
            prefix=out/f'{actual.stem}-{i+1}'
            px.save(str(prefix)+'-dotnet.png');py.save(str(prefix)+'-rust.png')
            if difference is not None:Image.fromarray(np.where(np.any(difference>0,axis=2)[:,:,None],np.array([255,0,0],dtype=np.uint8),xa//3+170)).save(str(prefix)+'-diff.png')
            (out/f'{actual.stem}-{i+1}-text.json').write_text(json.dumps({'dotnet':x.get_text('dict'),'rust':y.get_text('dict')},ensure_ascii=False,indent=2,default=lambda b:f'<{len(b)} binary bytes>'),encoding='utf-8')
    results.append(result)
(out/'results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
for r in results:print(r['name'], 'PASS' if r['equal'] else 'FAIL',f"pages {r['reference_pages']}/{r['rust_pages']}",[(p.get('changed_pixels'),p.get('mean_channel_difference'))for p in r['pages']])
raise SystemExit(0 if results and all(r['equal'] for r in results) else 1)
