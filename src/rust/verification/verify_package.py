"""Verify a runtime bundle with development native lookup disabled."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,subprocess,sys
root=Path(__file__).resolve().parents[1];os.chdir(root)
p=argparse.ArgumentParser();p.add_argument('package');p.add_argument('--reference',default='output/project');p.add_argument('--output');a=p.parse_args()
package=Path(a.package).resolve();manifest=json.loads((package/'manifest.json').read_text())
assert manifest['source_version']=='2026.9.1'and manifest['skia_abi']==26 and manifest['qpdf_abi']==4
for name,checksum in manifest['files'].items():assert hashlib.sha256((package/name).read_bytes()).hexdigest()==checksum,name
out=Path(a.output).resolve()if a.output else root/'output'/('package-verification-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True,exist_ok=False)
env={**os.environ,'QUESTPDF_DISABLE_DEVELOPMENT_LOOKUP':'1','QUESTPDF_TEST_NOW':'2026-10-24T12:00:00'}
for key in ['QUESTPDF_SKIA_PATH','QUESTPDF_QPDF_PATH','DOC_REPORTS_ASSETS_PATH']:env.pop(key,None)
def run(name,command,cwd=root):
    result=subprocess.run([str(x)for x in command],cwd=cwd,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace')
    (out/(name+'.log')).write_text(result.stdout,encoding='utf-8');print(result.stdout[-1800:],flush=True)
    if result.returncode:raise SystemExit(result.returncode)
if manifest['platform']=='win-x64':
    run('cli',[package/'receipt-preview.exe','--output',out/'cli','--images','--escpos'],out)
    run('ffi',[sys.executable,'verification/verify_ffi.py','--library',package/'ris_document_reports.dll','--fixtures',package/'fixtures','--output',out/'ffi'])
elif manifest['platform']=='linux-x64':
    image='rust:1-slim-bookworm@sha256:86234e857a413004348b471d11a4ebe1aa92d2eae2b6b143cac68fd1ac3c48be'
    mounted_output='/workspace/'+out.relative_to(root).as_posix()
    run('linux-bundle',['docker','run','--rm','--mount',f'type=bind,source={root},target=/workspace','--mount',f'type=bind,source={package},target=/package,readonly','--workdir','/tmp','--env','QUESTPDF_DISABLE_DEVELOPMENT_LOOKUP=1','--env','QUESTPDF_TEST_NOW=2026-10-24T12:00:00','--env','TZ=CET-1',image,'sh','/workspace/verification/test_linux_package.sh',mounted_output])
else:raise SystemExit('Unsupported bundle platform')
checks=[['compare_pdfs.py',a.reference,out/'cli','--output',out/'comparison-cli'],['compare_images.py',a.reference,out/'cli','--output',out/'comparison-images.json'],['compare_pdfs.py',a.reference,out/'ffi','--output',out/'comparison-ffi']]
for arguments in checks:run(arguments[0]+'-'+Path(str(arguments[2])).name,[sys.executable,'verification/'+arguments[0],*arguments[1:]])
print('Bundle verification PASSED: '+str(out))
