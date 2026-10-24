"""Differential RIS DateTime Kind/offset JSON cases, using unchanged C# templates."""
from pathlib import Path
import argparse,datetime,json,os,re,subprocess,sys
root=Path(__file__).resolve().parents[1];os.chdir(root)
p=argparse.ArgumentParser();p.add_argument('--fixtures',default='verification/fixtures');p.add_argument('--output');a=p.parse_args()
out=Path(a.output)if a.output else Path('output')/('json-verification-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True,exist_ok=False)
def transform(value,mode):
    if isinstance(value,dict):return{k:transform(v,mode)for k,v in value.items()}
    if isinstance(value,list):return[transform(v,mode)for v in value]
    if isinstance(value,str)and re.fullmatch(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?',value):
        if value.startswith('0001-'):return value # Preserve default DateTime.MinValue sentinels.
        clock=re.sub(r'(Z|[+-]\d{2}:\d{2})$','',value)
        if mode=='utc':return clock+'Z'
        if mode=='offset':return clock+'+02:00'
        return clock
    return value
cases=[]
for fixture in sorted(Path(a.fixtures).glob('*.json')):
    payload=json.loads(fixture.read_text())
    for mode in ['utc','offset','unspecified']:cases.append(dict(name=fixture.stem+'-'+mode,kind=fixture.stem,payload=transform(payload,mode)))
manifest=out/'cases.json';manifest.write_text(json.dumps(cases,indent=2),encoding='utf-8')
env={**os.environ,'QUESTPDF_TEST_NOW':'2026-10-24T12:00:00'}
for name,command in [('dotnet',['dotnet','run','--project','verification/json-oracle/JsonOracle.csproj','-p:WarningLevel=0','-p:NuGetAudit=false','--',manifest,out/'dotnet']),('rust',['cargo','run','--locked','-p','ris-document-server','--example','json_cases','--',manifest,out/'rust'])]:
    result=subprocess.run([str(x)for x in command],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace')
    (out/(name+'.log')).write_text(result.stdout,encoding='utf-8')
    if result.returncode:print(result.stdout);raise SystemExit(result.returncode)
result=subprocess.run([sys.executable,'verification/compare_pdfs.py',str(out/'dotnet'),str(out/'rust'),'--output',str(out/'comparison')])
print(out);raise SystemExit(result.returncode)
