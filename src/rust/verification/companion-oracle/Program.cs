using QuestPDF;
using QuestPDF.Companion;
using QuestPDF.Fluent;
using QuestPDF.Infrastructure;
Settings.License=LicenseType.Community;
var document=Document.Create(d=>d.Page(p=>{
 p.Size(240,180);p.Margin(10);p.DefaultTextStyle(s=>s.FontSize(10));
 p.Content().Column(c=>{
  c.Item().DebugPointer("Preview fixture").Border(1).Padding(4).Text("Companion API v3");
  c.Item().PageBreak();c.Item().Background("#bbddff").Text("Second page");
 });
 p.Footer().Text(t=>{t.Span("Page ");t.CurrentPageNumber();});
}));
using var cancellation=new CancellationTokenSource(TimeSpan.FromSeconds(4));
await document.ShowInCompanionAsync(int.Parse(args[0]),cancellation.Token);
