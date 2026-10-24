using QuestPDF;using QuestPDF.Fluent;using QuestPDF.Infrastructure;using QuestPDF.Helpers;
Settings.License=LicenseType.Community;
Directory.CreateDirectory(args[0]);
var document=Document.Create(c=>c.Page(p=>{
 p.Size(240,180);p.Margin(10);p.DefaultTextStyle(s=>s.FontSize(10));
 p.Content().Column(c=>{
  c.Spacing(5);c.Item().Text(t=>{t.Span("Superscript ");t.Span("2").Superscript();t.Span(" subscript ");t.Span("3").Subscript();});
  c.Item().Text("office affinity ffi fi fl").EnableFontFeature("liga");
  c.Item().Text("Decorated text").Underline().Overline().DecorationWavy().DecorationColor("#ff0000");
  c.Item().Width(110).Text("averylongwordwithoutspaces").BreakAnywhere();
  c.Item().Height(20).Background("#8088bbff");
 });
}));
document.GeneratePdf(Path.Combine(args[0],"text-styles.pdf"));
foreach(var (name,format,transparent,extension) in new[]{("opaque",ImageFormat.Png,false,"png"),("transparent",ImageFormat.Png,true,"png"),("jpeg",ImageFormat.Jpeg,true,"jpg"),("webp",ImageFormat.Webp,true,"webp")}){
 var images=document.GenerateImages(new ImageGenerationSettings{ImageFormat=format,UseTransparentBackground=transparent,RasterDpi=150}).ToArray();
 File.WriteAllBytes(Path.Combine(args[0],name+"."+extension),images.Single());
}
File.WriteAllText(Path.Combine(args[0],"page.svg"),document.GenerateSvg().Single());
// Independent evidence for the RIS decimal formatting contract.
File.WriteAllText(Path.Combine(args[0],"decimal.txt"),string.Join("\n",new[]{9007199254740993.125m,-2.345m,0.1m+0.2m}.Select(x=>x.ToString("N2",System.Globalization.CultureInfo.InvariantCulture))));
QuestPDF.Drawing.FontManager.RegisterFontFromFile("document-server/assets/BAHNSCHRIFT.TTF");
File.WriteAllText(Path.Combine(args[0],"fonts-registered.json"),System.Text.Json.JsonSerializer.Serialize(QuestPDF.Drawing.FontManager.GetRegisteredFonts()));
File.WriteAllText(Path.Combine(args[0],"fonts-system.json"),System.Text.Json.JsonSerializer.Serialize(QuestPDF.Drawing.FontManager.GetSystemFonts()));
Document.Create(c=>c.Page(p=>{p.Size(240,180);p.Margin(10);p.Content().Text("Application font registration").FontFamily("Bahnschrift").FontSize(12);})).GeneratePdf(Path.Combine(args[0],"font-management.pdf"));
