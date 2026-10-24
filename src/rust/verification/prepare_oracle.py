"""Build an independent .NET oracle from unchanged RIS source files.

Copies live source into ignored artifacts; freezes only wall-clock expressions.
Exports original ReceiptPreview inputs rather than reconstructing fixtures in Rust.
"""
from pathlib import Path
import argparse, json, re, hashlib

p=argparse.ArgumentParser()
p.add_argument('--source',default=r'D:\WebProjects\50lab-ris\50lab-ris\DocumentServer')
p.add_argument('--questpdf',choices=['project','checkout'],default='project')
a=p.parse_args()
root=Path(__file__).resolve().parents[1]
source=Path(a.source)
out=root/'verification'/'artifacts'/('oracle-'+a.questpdf)
out.mkdir(parents=True,exist_ok=True)
manifest={}
files=list((source/'DocumentServer'/'Documents').glob('*.cs'))+list((source/'DocumentServer'/'Models').glob('*.cs'))+[source/'DocumentServer'/'Native'/n for n in ['ReportModels.cs','InvoiceReceiptModels.cs','LargeReceiptModels.cs']]
for file in files:
    text=file.read_text(encoding='utf-8-sig')
    manifest[str(file)]=hashlib.sha256(file.read_bytes()).hexdigest()
    text=text.replace('DateTimeOffset.Now','Frozen.Offset').replace('DateTime.Now','Frozen.Now')
    (out/file.name).write_text(text,encoding='utf-8')
preview=(source/'ReceiptPreview'/'Program.cs').read_text(encoding='utf-8-sig')
methods=preview[preview.index('static InvoiceReceiptModel CreateFakeReceipt()'):]
program='''using System.Text.Json;
using System.Globalization;
using QuestPDF;
using QuestPDF.Drawing;
using QuestPDF.Fluent;
using QuestPDF.Helpers;
using QuestPDF.Infrastructure;
using RisDocumentServer.Documents;
using RisDocumentServer.Models;
using ESCPOS_NET.Emitters;
Settings.License = LicenseType.Community;
CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
foreach(var font in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory,"Assets"),"*.TTF"))
  using(var f=File.OpenRead(font)) FontManager.RegisterFont(f);
var output=args.Length>0?args[0]:"output";
Directory.CreateDirectory(output);
var samples=new (string name,object model,IDocument document)[]{
 ("receipt",CreateFakeReceipt(),new InvoiceReceiptDocument(CreateFakeReceipt())),
 ("debt-receipt",CreateFakeDebtReceipt(),new InvoiceReceiptDocument(CreateFakeDebtReceipt())),
 ("large-receipt",CreateFakeLargeReceipt(),new LargeReceiptDocument(CreateFakeLargeReceipt())),
 ("debt-report",CreateFakeDebtReport(),new DebtDocument(CreateFakeDebtReport())),
 ("debt-out-report",CreateFakeDebtOutReport(),new DebtOutReportByUsers(CreateFakeDebtOutReport())),
 ("income-report",CreateFakeIncomeReport(),new IncomeReportByUsers(CreateFakeIncomeReport())),
 ("substitute-report",CreateFakeSubstituteReport(),new SubstituteDoctorMonthlyReport(CreateFakeSubstituteReport()))
};
foreach(var (name,model,document) in samples){
 File.WriteAllText(Path.Combine(output,name+".json"),JsonSerializer.Serialize(model,model.GetType(),new JsonSerializerOptions{WriteIndented=true}));
 document.GeneratePdf(Path.Combine(output,name+".pdf"));
 var images=document.GenerateImages(new ImageGenerationSettings{RasterDpi=150}).ToArray();
 for(var i=0;i<images.Length;i++)File.WriteAllBytes(Path.Combine(output,name+"-"+(i+1)+".png"),images[i]);
 if(name is "receipt" or "debt-receipt" or "large-receipt"){
  var epson=new EPSON();epson.Initialize();
  var bytes=epson.LeftAlign().Concat(epson.BufferImage(images.Single(),isLegacy:true,maxWidth:576,color:0))
    .Concat(epson.SetImageDensity(true)).Concat(epson.WriteImageFromBuffer()).Concat(epson.FeedLines(6)).Concat(epson.FullCut()).ToArray();
  File.WriteAllBytes(Path.Combine(output,name+"-escpos.bin"),bytes);
 }
 Console.WriteLine(name);
}
'''+methods+'''
static class Frozen {
 public static DateTime Now => new(2026,10,24,12,0,0,DateTimeKind.Local);
 public static DateTimeOffset Offset => new(Now);
}
'''
(out/'Program.cs').write_text(program,encoding='utf-8')
project=(source/'DocumentServer/DocumentServer.csproj').read_text(encoding='utf-8-sig')
version=re.search(r'<PackageReference Include="QuestPDF" Version="([^"]+)"',project)[1]
quest=(f'<PackageReference Include="QuestPDF" Version="{version}" />' if a.questpdf=='project' else f'<ProjectReference Include="{root.parent / "dotnet/library/QuestPDF/QuestPDF.csproj"}" />')
(out/'Oracle.csproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
<PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings><Nullable>enable</Nullable></PropertyGroup>
<ItemGroup>{quest}<PackageReference Include="ZXing.Net" Version="0.16.11" /><PackageReference Include="ESCPOS_NET" Version="3.0.0" /><PackageReference Include="SixLabors.ImageSharp" Version="2.1.13" /></ItemGroup>
<ItemGroup><Content Include="{source/'DocumentServer/Assets/*'}" Link="Assets/%(Filename)%(Extension)" CopyToOutputDirectory="PreserveNewest" />
<Content Include="{source/'Fonts/BAHNSCHRIFT.TTF'}" Link="Assets/BAHNSCHRIFT.TTF" CopyToOutputDirectory="PreserveNewest" /></ItemGroup>
</Project>''',encoding='utf-8')
(out/'source-hashes.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print(out/'Oracle.csproj')
