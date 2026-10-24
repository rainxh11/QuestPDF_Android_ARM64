"""Translate stored model properties (not computed properties) from RIS C# models."""
from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
source=Path(r'D:\WebProjects\50lab-ris\50lab-ris\DocumentServer\DocumentServer')
files=[source/'Native'/f for f in ['ReportModels.cs','InvoiceReceiptModels.cs','LargeReceiptModels.cs']]+[source/'Models/SubstituteDoctorReportModel.cs']
def snake(s):return re.sub(r'(?<!^)(?=[A-Z])','_',s).lower()
def typ(s):
    if s.endswith('?'):return 'Option<'+typ(s[:-1])+'>'
    if s.endswith('[]'):return 'Vec<'+typ(s[:-2])+'>'
    if s.startswith('List<'):return 'Vec<'+typ(s[5:-1])+'>'
    return {'string':'String','bool':'bool','int':'i32','decimal':'Decimal','double':'f64','DateTime':'Date'}.get(s,s)
out=['// Stored properties ported from the RIS C# models.','use serde::{Deserialize,Serialize};','use crate::util::Date;','use rust_decimal::Decimal;']
fields=set()
for file in files:
    text=file.read_text(encoding='utf-8-sig')
    for m in re.finditer(r'public (?:sealed )?class (\w+)\s*\{',text):
        begin=m.end();depth=1;i=begin
        while depth:
            depth+=(text[i]=='{')-(text[i]=='}');i+=1
        body=text[begin:i-1]
        out+=['#[derive(Clone,Debug,Default,Deserialize,Serialize)]','#[serde(default)]',f'pub struct {m[1]} '+'{']
        for prop in re.finditer(r'public ([\w<>?\[\]]+) (\w+)\s*\{\s*get;\s*set;\s*\}',body):
            t=typ(prop[1]);field=snake(prop[2])
            fields.add(field)
            if t.startswith('Vec<'):out+=[' #[serde(deserialize_with="crate::util::null_default")]']
            if t=='Decimal':out+=[' #[serde(with="crate::util::decimal_json")]']
            out+=[f' pub {field}:{t},']
        out+=['}']
out+=['pub const JSON_FIELDS:&[&str]=&['+','.join('"'+f+'"'for f in sorted(fields))+'];']
(root/'document-server/src/models.rs').write_text('\n'.join(out)+'\n',encoding='utf-8')
