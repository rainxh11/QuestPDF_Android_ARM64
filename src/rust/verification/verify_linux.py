"""Build/run Rust in Linux Docker, then compare to the independent C# outputs."""
from pathlib import Path
import argparse, datetime, json, os, shutil, subprocess, sys
ROOT=Path(__file__).resolve().parents[1]
IMAGE='rust:1-slim-bookworm@sha256:86234e857a413004348b471d11a4ebe1aa92d2eae2b6b143cac68fd1ac3c48be'
p=argparse.ArgumentParser();p.add_argument('--reference',default='output/project');p.add_argument('--engine-reference',default='output/engine-dotnet');p.add_argument('--output');a=p.parse_args()
out=Path(a.output)if a.output else Path('output')/('linux-verification-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
os.chdir(ROOT)
cases=json.loads((ROOT/'verification/engine-cases.json').read_text())
# System font availability is host-dependent. This fixture explicitly requires
# Windows Arial; all other engine fixtures use the embedded/registered fonts.
cases=[case for case in cases if not case.get('system_fonts')]
out.mkdir(parents=True,exist_ok=False)
(out/'engine-dotnet').mkdir()
for case in cases:
    for extension in ['pdf','error']:
        source=Path(a.engine_reference)/(case['name']+'.'+extension)
        if source.exists():shutil.copy2(source,out/'engine-dotnet'/source.name)
print('Excluding the Windows Arial system-font fixture from Linux comparison.',flush=True)
(out/'engine-cases.json').write_text(json.dumps(cases),encoding='utf-8')
command=['docker','run','--rm','--mount',f'type=bind,source={ROOT},target=/workspace','--mount',f'type=bind,source={(ROOT.parent/"dotnet/library/QuestPDF/Runtimes/linux-x64/native").resolve()},target=/native,readonly','--workdir','/workspace']
for key,value in dict(CARGO_TARGET_DIR='/workspace/target/linux',QUESTPDF_SKIA_PATH='/native/libquestpdf_skia.so',QUESTPDF_QPDF_PATH='/native/libquestpdf_qpdf.so',QUESTPDF_TEST_NOW='2026-10-24T12:00:00',TZ='CET-1').items():command+=['--env',key+'='+value]
command += [IMAGE,'sh','verification/build_linux.sh',out.as_posix()]
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace')
(out/'build.log').write_text(result.stdout,encoding='utf-8');print(result.stdout[-2400:])
if result.returncode:raise SystemExit(result.returncode)
checks=[['compare_pdfs.py',a.reference,out/'rust','--output',out/'comparison-ris'],['compare_images.py',a.reference,out/'rust','--output',out/'comparison-images.json'],['verify_operations.py',out/'operations'],['compare_pdfs.py',out/'engine-dotnet',out/'engine-rust','--output',out/'comparison-engine','--cases',out/'engine-cases.json'],['compare_pdfs.py',a.reference,out/'ffi','--output',out/'comparison-ffi']]
passed=True
for arguments in checks:
    passed &= subprocess.run([sys.executable,'verification/'+arguments[0],*[str(x)for x in arguments[1:]]]).returncode==0
print(('Linux verification PASSED: 'if passed else'Linux verification FAILED: ')+str(out))
raise SystemExit(0 if passed else 1)
