"""Compare logical PDF trees, annotation descriptions, bounds and conformance IDs."""
from pathlib import Path
import argparse,json
from pypdf import PdfReader
from pypdf.generic import IndirectObject,BooleanObject,NullObject
from xml.etree import ElementTree as ET
p=argparse.ArgumentParser();p.add_argument('reference');p.add_argument('actual');p.add_argument('--cases',default='verification/engine-cases.json');p.add_argument('--output',default='output/comparison-semantics.json');a=p.parse_args()
def describe(path):
    r=PdfReader(path);pages={(p.indirect_reference.idnum,p.indirect_reference.generation):i+1 for i,p in enumerate(r.pages)}
    def normalized(obj,key=None):
        if isinstance(obj,IndirectObject):
            if key=='/Pg':return pages[(obj.idnum,obj.generation)]
            obj=obj.get_object()
        if isinstance(obj,dict):return{k:normalized(v,k)for k,v in obj.items()if k not in ['/P','/Parent','/StructParent']}
        if isinstance(obj,list):return[normalized(v)for v in obj]
        if isinstance(obj,BooleanObject):return obj.value
        if isinstance(obj,NullObject):return None
        if isinstance(obj,float):return round(float(obj),5)
        return obj
    root=r.trailer['/Root'];structure=root.get('/StructTreeRoot')
    xmp=r.xmp_metadata
    import pymupdf
    with pymupdf.open(path)as d:xml=d.get_xml_metadata()
    metadata=ET.fromstring(xml)if xml else None
    ids={}
    if metadata is not None:
        for ns in ['http://www.aiim.org/pdfa/ns/id/','http://www.aiim.org/pdfua/ns/id/']:
            for tag in ['part','conformance','rev']:
                value=metadata.find('.//{'+ns+'}'+tag)
                if value is not None:ids[ns+tag]=value.text
    return dict(language=root.get('/Lang'),marked=normalized(root.get('/MarkInfo')),structure=normalized(structure.get_object()['/K'])if structure else None,conformance=ids)
def equal(x,y):
    if isinstance(x,(float,int))and isinstance(y,(float,int)):return abs(x-y)<=.001
    if type(x)!=type(y):return x==y
    if isinstance(x,dict):return x.keys()==y.keys()and all(equal(x[k],y[k])for k in x)
    if isinstance(x,list):return len(x)==len(y)and all(equal(u,v)for u,v in zip(x,y))
    return x==y
results=[]
for case in json.loads(Path(a.cases).read_text()):
    if not case.get('pdfua')and not case.get('pdfa'):continue
    name=case['name'];reference=describe(Path(a.reference)/(name+'.pdf'));actual=describe(Path(a.actual)/(name+'.pdf'))
    result=dict(name=name,equal=equal(reference,actual),reference=reference,actual=actual);results.append(result)
    print(name,'PASS'if result['equal']else'FAIL','logical tree, annotations, bounds, language and conformance IDs')
Path(a.output).write_text(json.dumps(results,indent=2,ensure_ascii=False),encoding='utf-8')
raise SystemExit(0 if results and all(r['equal']for r in results)else 1)
