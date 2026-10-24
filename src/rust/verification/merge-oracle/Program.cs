using QuestPDF;
using QuestPDF.Fluent;
using QuestPDF.Infrastructure;
Settings.License=LicenseType.Community;
Settings.UseSystemFonts=false;
Directory.CreateDirectory(args[0]);
foreach(var continuous in new[]{false,true}) foreach(var tagged in new[]{false,true}) {
 var name=$"merged-{continuous.ToString().ToLowerInvariant()}-{tagged.ToString().ToLowerInvariant()}";
 var merged=Document.Merge(Part("Part A",2),Part("Part B",3));
 if(continuous)merged.UseContinuousPageNumbers();else merged.UseOriginalPageNumbers();
 merged.WithSettings(new DocumentSettings{PDFUA_Conformance=tagged?PDFUA_Conformance.PDFUA_1:PDFUA_Conformance.None})
  .WithMetadata(new DocumentMetadata{Title=name,Language="en-US"}).GeneratePdf(Path.Combine(args[0],name+".pdf"));
}
static Document Part(string name,int count)=>Document.Create(d=>d.Page(p=>{
 p.Size(240,200);p.Margin(10);p.DefaultTextStyle(s=>s.FontSize(10));
 p.Header().Text(name).Bold();
 p.Content().Section("shared").Column(c=>{
  for(var i=0;i<count;i++){
   if(i>0)c.Item().PageBreak();
   c.Item().Text($"{name}, item {i+1}");
   c.Item().SemanticLink("Go to section start").SectionLink("shared").Text("Go to section start");
  }
 });
 p.Footer().Text(t=>{
  t.CurrentPageNumber();t.Span(" / ");t.TotalPages();
  t.Span("; section ");t.BeginPageNumberOfSection("shared");t.Span(" - ");t.EndPageNumberOfSection("shared");
 });
}));
