"""Rebuild and independently verify the port in a fresh, uniquely named output.

Build/generation errors stop immediately. Comparisons are all collected, and any
difference makes the overall exit code nonzero. Existing output is never deleted.
"""
from pathlib import Path
import argparse,datetime,json,os,re,shutil,subprocess,sys,uuid
root=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--source',default=r'D:\WebProjects\50lab-ris\50lab-ris\DocumentServer');p.add_argument('--output');p.add_argument('--pdf-only',action='store_true',help='Skip the separate native image comparison');p.add_argument('--verapdf-jar',default='verification/artifacts/verapdf-tools/runtime-1.30.3/bin/cli-1.30.3.jar');a=p.parse_args()
os.chdir(root)
out=Path(a.output).resolve()if a.output else root/'output'/('verification-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')+'-'+uuid.uuid4().hex[:6])
out.mkdir(parents=True,exist_ok=False)
env={**os.environ,'QUESTPDF_TEST_NOW':'2026-10-24T12:00:00'}
rust_bin=Path.home()/'scoop/apps/rust/current/bin'
if rust_bin.exists():env['PATH']=str(rust_bin)+os.pathsep+env['PATH']
steps=[]
def run(name,command,required=True):
    print(name,flush=True)
    result=subprocess.run([str(x)for x in command],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace')
    (out/(name+'.log')).write_text(result.stdout,encoding='utf-8');print(result.stdout[-1800:],flush=True)
    steps.append(dict(name=name,exit_code=result.returncode,command=[str(x)for x in command]))
    (out/'steps.json').write_text(json.dumps(steps,indent=2),encoding='utf-8')
    if required and result.returncode:raise SystemExit(f'{name} failed; see {out}')
    return result.returncode==0
cs=Path(a.source)
version=re.search(r'<Version>([^<]+)</Version>',(root.parent/'dotnet/library/QuestPDF/QuestPDF.csproj').read_text())[1]
for project in [cs/'DocumentServer/DocumentServer.csproj',cs/'ReceiptPreview/ReceiptPreview.csproj']:
    assert re.search(r'Include="QuestPDF" Version="([^"]+)"',project.read_text(encoding='utf-8-sig'))[1]==version,(project,'version mismatch')
(out/'baseline.json').write_text(json.dumps(dict(questpdf_version=version,rust_platform=sys.platform,source=str(cs),frozen_now=env['QUESTPDF_TEST_NOW']),indent=2))
py=sys.executable
run('format',['cargo','fmt','--all','--','--check'])
run('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'])
run('tests',['cargo','test','--workspace','--locked'])
run('build',['cargo','build','--workspace','--locked'])
for baseline in ['project','checkout']:
    run('prepare-'+baseline,[py,'verification/prepare_oracle.py','--source',cs,'--questpdf',baseline])
    run('oracle-'+baseline,['dotnet','run','--project',f'verification/artifacts/oracle-{baseline}/Oracle.csproj','-p:WarningLevel=0','-p:NuGetAudit=false','--',out/baseline])
run('rust-previews',['cargo','run','--locked','-p','receipt-preview','--','--fixtures',out/'project','--output',out/'rust','--images','--escpos'])
library='ris_document_reports.dll'if os.name=='nt'else'libris_document_reports.dylib'if sys.platform=='darwin'else'libris_document_reports.so'
run('ffi',[py,'verification/verify_ffi.py','--library',root/'target/debug'/library,'--fixtures',out/'project','--output',out/'ffi','--printer-reference',out/'project'])
run('engine-fixtures',[py,'verification/make_engine_fixtures.py'])
run('engine-dotnet',['dotnet','run','--project','verification/engine-oracle/EngineOracle.csproj','-p:WarningLevel=0','--','verification/engine-cases.json',out/'engine-dotnet'])
run('engine-rust',['cargo','run','--locked','-p','questpdf','--example','engine_cases','--','verification/engine-cases.json',out/'engine-rust'])
run('merged-dotnet',['dotnet','run','--project','verification/merge-oracle/MergeOracle.csproj','-p:WarningLevel=0','--',out/'merged-dotnet'])
run('merged-rust',['cargo','run','--locked','-p','questpdf','--example','merged_documents','--',out/'merged-rust'])
run('exports-dotnet',['dotnet','run','--project','verification/export-oracle/ExportOracle.csproj','-p:WarningLevel=0','--',out/'exports-dotnet'])
run('exports-rust',['cargo','run','--locked','-p','questpdf','--example','export_formats','--',out/'exports-rust'])
run('settings-dotnet',['dotnet','run','--project','verification/settings-oracle/SettingsOracle.csproj','-p:WarningLevel=0','--',out/'settings-dotnet'])
run('settings-rust',['cargo','run','--locked','-p','questpdf','--example','settings_behavior','--',out/'settings-rust'])
run('operations',['cargo','run','--locked','-p','questpdf','--example','pdf_operations','--',out/'operations'])
run('json-cases',[py,'verification/verify_json_cases.py','--fixtures',out/'project','--output',out/'json-cases'])
if shutil.which('postman'):run('companion',[py,'verification/verify_companion.py','--output',out/'companion'])
else:print('Companion protocol verification SKIPPED: install Postman CLI >=1.70 for the local simulation.',flush=True)
checks=[]
for reference,actual,name in [('project','rust','ris-package'),('checkout','rust','ris-checkout'),('project','ffi','ris-ffi'),('engine-dotnet','engine-rust','engine'),('merged-dotnet','merged-rust','merged'),('exports-dotnet','exports-rust','text-styles'),('settings-dotnet','settings-rust','settings')]:
    command=[py,'verification/compare_pdfs.py',out/reference,out/actual,'--output',out/('compare-'+name)]
    if name=='engine':command+=['--cases','verification/engine-cases.json']
    checks.append(run('compare-'+name,command,False))
checks.append(run('verify-exports',[py,'verification/verify_exports.py',out/'exports-dotnet',out/'exports-rust'],False))
checks.append(run('verify-settings',[py,'verification/compare_images.py',out/'settings-dotnet',out/'settings-rust','--output',out/'settings-images.json'],False))
checks.append(run('verify-operations',[py,'verification/verify_operations.py',out/'operations'],False))
checks.append(run('compare-semantics',[py,'verification/compare_semantics.py',out/'engine-dotnet',out/'engine-rust','--output',out/'semantics.json'],False))
checks.append(run('compare-merged-semantics',[py,'verification/compare_semantics.py',out/'merged-dotnet',out/'merged-rust','--cases','verification/merge-cases.json','--output',out/'merged-semantics.json'],False))
if Path(a.verapdf_jar).is_file():
    for baseline in ['engine-dotnet','engine-rust']:checks.append(run('conformance-'+baseline,[py,'verification/verify_conformance.py',out/baseline,'--jar',a.verapdf_jar,'--output',out/('conformance-'+baseline+'.json')],False))
    for baseline in ['merged-dotnet','merged-rust']:checks.append(run('conformance-'+baseline,[py,'verification/verify_conformance.py',out/baseline,'--jar',a.verapdf_jar,'--cases','verification/merge-cases.json','--output',out/('conformance-'+baseline+'.json')],False))
else:print('veraPDF conformance validation SKIPPED: supply --verapdf-jar (see README).',flush=True)
if not a.pdf_only:checks.append(run('compare-native-images',[py,'verification/compare_images.py',out/'project',out/'rust','--output',out/'image-comparison.json'],False))
print('Verification '+('PASSED'if all(checks)else'FAILED')+': '+str(out))
raise SystemExit(0 if all(checks)else 1)
