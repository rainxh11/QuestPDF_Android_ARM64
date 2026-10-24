"""Independent veraPDF machine validation of the archival/accessibility corpus."""
from pathlib import Path
import argparse,subprocess,json
p=argparse.ArgumentParser();p.add_argument('directory');p.add_argument('--jar',default='verification/artifacts/verapdf-tools/runtime-1.30.3/bin/cli-1.30.3.jar');p.add_argument('--cases',default='verification/engine-cases.json');p.add_argument('--output',default='output/conformance.json');a=p.parse_args()
cases=[c for c in json.loads(Path(a.cases).read_text())if c.get('pdfa')or c.get('pdfua')]
files=[str(Path(a.directory)/(c['name']+'.pdf'))for c in cases]
result=subprocess.run(['java','-jar',str(Path(a.jar)),'--format','json',*files],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding='utf-8',errors='replace')
Path(a.output).write_text(result.stdout,encoding='utf-8')
if not result.stdout.strip():print(result.stderr);raise SystemExit(result.returncode or 1)
report=json.loads(result.stdout)['report'];jobs=report['jobs'];assert len(jobs)==len(files)
passed=True
for job in jobs:
    for validation in job.get('validationResult',[]):
        print(Path(job['itemDetails']['name']).name,validation['profileName'],'PASS'if validation['compliant']else'FAIL',validation['details']['failedRules'],'failed rules')
        passed &= validation['compliant']
    if not job.get('validationResult'):passed=False
if result.stderr:print(result.stderr)
raise SystemExit(0 if passed and result.returncode==0 else 1)
