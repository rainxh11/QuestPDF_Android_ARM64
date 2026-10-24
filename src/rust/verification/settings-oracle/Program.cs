using QuestPDF;
using QuestPDF.Fluent;
using QuestPDF.Infrastructure;
Directory.CreateDirectory(args[0]);
Settings.FontDiscoveryPath="document-server/assets";
try{Doc().GeneratePdf();throw new Exception("Missing license declaration was accepted");}catch(Exception e)when(e.Message.Contains("configure the QuestPDF license")){}
Settings.License=LicenseType.Evaluation;
Doc().GeneratePdf(Path.Combine(args[0],"evaluation.pdf"));
Settings.License=LicenseType.Community;
Doc().GeneratePdf(Path.Combine(args[0],"discovery.pdf"));
Settings.EnableCaching=false;
Doc().GeneratePdf(Path.Combine(args[0],"uncached.pdf"));
Settings.ThrowOnMissingFontFamilies=false;Settings.ThrowOnMissingTextGlyphs=false;
Document.Create(d=>d.Page(p=>{p.Size(240,180);p.Margin(10);p.Content().Text("Missing font fallback").FontFamily("Unregistered Fixture Font").FontSize(12);})).GeneratePdf(Path.Combine(args[0],"missing-font.pdf"));
var invalid=Document.Create(d=>d.Page(p=>{p.Size(100,100);p.Content().DebugPointer("fixture overflow label").Width(200).Text("Too wide");}));
foreach(var detail in new[]{false,true}){
 Settings.EnableDetailedLayoutErrors=detail;
 try{invalid.GeneratePdf();throw new Exception("Invalid layout was accepted");}catch(QuestPDF.Drawing.Exceptions.DocumentLayoutException e){if(e.Message.Contains("fixture overflow label")!=detail)throw;}
}
var dynamic=Document.Create(d=>d.Page(p=>{p.Size(240,180);p.Margin(10);p.Content().Height(60).Image((QuestPDF.Elements.GenerateDynamicImageDelegate)(payload=>{if(payload.Dpi!=150)throw new Exception("Unexpected callback DPI");return File.ReadAllBytes("verification/artifacts/compression-source.png");}));}));
File.WriteAllBytes(Path.Combine(args[0],"dynamic.png"),dynamic.GenerateImages(new ImageGenerationSettings{RasterDpi=150}).Single());
File.Copy("verification/artifacts/compression-source.png",Path.Combine(AppContext.BaseDirectory,"relative-resource.png"),true);
Document.Create(d=>d.Page(p=>{
 p.Size(240,180);p.Margin(10);p.Content().Column(c=>{
  c.Item().Width(80).Image("relative-resource.png");
  c.Item().Svg("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"80\" height=\"40\"><image width=\"80\" height=\"40\" xlink:href=\"relative-resource.png\"/></svg>");
 });
})).GeneratePdf(Path.Combine(args[0],"relative-resources.pdf"));
static Document Doc()=>Document.Create(d=>d.Page(p=>{p.Size(240,180);p.Margin(10);p.Content().Text("Automatic application font discovery").FontFamily("Bahnschrift").FontSize(12);}));
