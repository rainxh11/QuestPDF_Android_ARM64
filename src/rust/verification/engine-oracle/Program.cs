using System.Text.Json;
using QuestPDF;
using QuestPDF.Fluent;
using QuestPDF.Infrastructure;
using QuestPDF.Elements.Table;
using QuestPDF.Helpers;
Settings.License = LicenseType.Community;
var cases = JsonDocument.Parse(File.ReadAllText(args[0])).RootElement;
// Font fallback is process-wide; configure it before the paragraph-builder pool is populated.
Settings.UseSystemFonts=cases.EnumerateArray().Any(test=>test.TryGetProperty("system_fonts",out var enabled)&&enabled.GetBoolean());
Directory.CreateDirectory(args[1]);
foreach (var test in cases.EnumerateArray())
{
 var name=test.GetProperty("name").GetString()!;
 File.Delete(Path.Combine(args[1],name+".pdf"));File.Delete(Path.Combine(args[1],name+".error"));
 try {
  var document=Document.Create(c=>c.Page(page=>{
   var size=test.GetProperty("size");
   if(test.TryGetProperty("continuous",out var continuous)&&continuous.GetBoolean())page.ContinuousSize(size[0].GetSingle());else page.Size(size[0].GetSingle(),size[1].GetSingle());
   if(test.TryGetProperty("min_size",out var min))page.MinSize(new PageSize(min[0].GetSingle(),min[1].GetSingle()));
   if(test.TryGetProperty("max_size",out var max))page.MaxSize(new PageSize(max[0].GetSingle(),max[1].GetSingle()));
   page.Margin(10);page.DefaultTextStyle(s=>s.FontSize(10));
   if(test.TryGetProperty("rtl",out var rtl)&&rtl.GetBoolean())page.ContentFromRightToLeft();
   if(test.TryGetProperty("header",out var header))Compose(header,page.Header());
   Compose(test.GetProperty("content"),page.Content());
   page.Footer().AlignCenter().Text(t=>{t.Span("Page ");t.CurrentPageNumber();t.Span(" of ");t.TotalPages();});
  }));
  var settings=new DocumentSettings();
  if(test.TryGetProperty("global_rtl",out var globalRtl)&&globalRtl.GetBoolean())settings.ContentDirection=ContentDirection.RightToLeft;
  if(test.TryGetProperty("pdfa",out var pdfa))settings.PDFA_Conformance=Enum.Parse<PDFA_Conformance>(pdfa.GetString()!);
  if(test.TryGetProperty("pdfua",out var pdfua)&&pdfua.GetBoolean())settings.PDFUA_Conformance=PDFUA_Conformance.PDFUA_1;
  document.WithSettings(settings).WithMetadata(new DocumentMetadata{Title=name,Language="en-US"}).GeneratePdf(Path.Combine(args[1],name+".pdf"));
  Console.WriteLine(name);
 }catch(Exception e){File.Delete(Path.Combine(args[1],name+".pdf"));File.WriteAllText(Path.Combine(args[1],name+".error"),e.GetType().Name+": "+e.Message);Console.WriteLine(name+" ERROR");}
}
static string S(JsonElement n,string key,string fallback="")=>n.TryGetProperty(key,out var v)?v.GetString()??fallback:fallback;
static float F(JsonElement n,string key,float fallback=0)=>n.TryGetProperty(key,out var v)?v.GetSingle():fallback;
static void Compose(JsonElement n,IContainer c)
{
 var kind=S(n,"kind");
 switch(kind){
  case "empty":break;
  case "text":{
   var t=c.Text(S(n,"text"));t.FontSize(F(n,"size",10));
   if(n.TryGetProperty("bold",out var b)&&b.GetBoolean())t.Bold();
   if(n.TryGetProperty("line_height",out var l))t.LineHeight(l.GetSingle());
   if(n.TryGetProperty("font_family",out var family))t.FontFamily(family.GetString()!);
   if(n.TryGetProperty("clamp",out var clamp))t.ClampLines(clamp.GetInt32(),S(n,"ellipsis","…"));
   if(n.TryGetProperty("paragraph_spacing",out var spacing))t.ParagraphSpacing(spacing.GetSingle());
   if(n.TryGetProperty("indent",out var indent))t.ParagraphFirstLineIndentation(indent.GetSingle());
   switch(S(n,"align")){case "center":t.AlignCenter();break;case "right":t.AlignRight();break;case "justify":t.Justify();break;}
   break;}
  case "column":c.Column(d=>{d.Spacing(F(n,"spacing"));foreach(var item in n.GetProperty("items").EnumerateArray())Compose(item,d.Item());});break;
  case "lazy":c.Lazy(child=>Compose(n.GetProperty("child"),child));break;
  case "dynamic":c.Dynamic(new DynamicRows((int)F(n,"count",30)));break;
  case "position_probe":c.Dynamic(new PositionProbe(S(n,"name")));break;
  case "grid":c.Grid(d=>{
   d.Columns((int)F(n,"count",12));d.Spacing(F(n,"spacing"));
   switch(S(n,"align")){case "center":d.AlignCenter();break;case "right":d.AlignRight();break;}
   foreach(var item in n.GetProperty("items").EnumerateArray())Compose(item.GetProperty("content"),d.Item((int)F(item,"span",1)));
  });break;
  case "dynamic_svg":c.Svg(size=>S(n,"text"));break;
  case "dynamic_image":{
   var image=c.Image((QuestPDF.Elements.GenerateDynamicImageDelegate)(payload=>{
    if(n.TryGetProperty("dpi",out var expected)&&payload.Dpi!=expected.GetInt32())throw new Exception("Unexpected image DPI");
    return Convert.FromBase64String(S(n,"data"));
   }));
   if(n.TryGetProperty("dpi",out var dpi))image.WithRasterDpi(dpi.GetInt32());
   if(n.TryGetProperty("quality",out var quality))image.WithCompressionQuality(Enum.Parse<ImageCompressionQuality>(quality.GetString()!));
   if(n.TryGetProperty("original",out var original))image.UseOriginalImage(original.GetBoolean());
   break;}
  case "image":{
   var image=c.Image(S(n,"path"));
   if(n.TryGetProperty("dpi",out var dpi))image.WithRasterDpi(dpi.GetInt32());
   if(n.TryGetProperty("quality",out var quality))image.WithCompressionQuality(Enum.Parse<ImageCompressionQuality>(quality.GetString()!));
   if(n.TryGetProperty("original",out var original))image.UseOriginalImage(original.GetBoolean());
   if(n.TryGetProperty("stretch",out var stretch)&&stretch.GetBoolean())image.FitUnproportionally();
   break;
  }
  case "inlined":c.Inlined(d=>{
   d.HorizontalSpacing(F(n,"horizontal"));d.VerticalSpacing(F(n,"vertical"));
   switch(S(n,"align")){case "left":d.AlignLeft();break;case "right":d.AlignRight();break;case "center":d.AlignCenter();break;case "justify":d.AlignJustify();break;case "around":d.AlignSpaceAround();break;}
   switch(S(n,"baseline")){case "middle":d.BaselineMiddle();break;case "bottom":d.BaselineBottom();break;}
   foreach(var item in n.GetProperty("items").EnumerateArray())Compose(item,d.Item());
  });break;
  case "rich_text":c.Text(t=>{
   foreach(var item in n.GetProperty("items").EnumerateArray()){
    switch(S(item,"kind")){
     case "inline":Compose(item.GetProperty("content"),t.Element(Enum.Parse<TextInjectedElementAlignment>(S(item,"alignment","AboveBaseline"))));break;
     case "link":t.Hyperlink(S(item,"text"),S(item,"url")).Underline();break;
     case "section_link":t.SectionLink(S(item,"text"),S(item,"name")).Underline();break;
     case "page_number":{
      var p=S(item,"type") switch {"begin"=>t.BeginPageNumberOfSection(S(item,"name")),"end"=>t.EndPageNumberOfSection(S(item,"name")),"within"=>t.PageNumberWithinSection(S(item,"name")),"section_total"=>t.TotalPagesWithinSection(S(item,"name")),"total"=>t.TotalPages(),_=>t.CurrentPageNumber()};
      if(S(item,"format")=="padded")p.Format(x=>x?.ToString("D3")??"000");
      break;
     }
     default:t.Span(S(item,"text"));break;
    }
   }
  });break;
  case "row":c.Row(d=>{d.Spacing(F(n,"spacing"));foreach(var item in n.GetProperty("items").EnumerateArray()){var child=S(item,"type")=="constant"?d.ConstantItem(F(item,"value")):S(item,"type")=="auto"?d.AutoItem():d.RelativeItem(F(item,"value",1));Compose(item.GetProperty("content"),child);}});break;
  case "padding":{var p=n.GetProperty("value");Compose(n.GetProperty("child"),c.PaddingLeft(p[0].GetSingle()).PaddingTop(p[1].GetSingle()).PaddingRight(p[2].GetSingle()).PaddingBottom(p[3].GetSingle()));break;}
  case "width":Compose(n.GetProperty("child"),c.Width(F(n,"value")));break;
  case "section":Compose(n.GetProperty("child"),c.Section(S(n,"name")));break;
  case "z_index":Compose(n.GetProperty("child"),c.ZIndex((int)F(n,"value")));break;
  case "capture":Compose(n.GetProperty("child"),c.CaptureContentPosition(S(n,"name")));break;
  case "semantic":{
   var container=S(n,"role") switch {
    "Art"=>c.SemanticArticle(),"Sect"=>c.SemanticSection(),"Div"=>c.SemanticDivision(),"H1"=>c.SemanticHeading1(),"H2"=>c.SemanticHeading2(),"P"=>c.SemanticParagraph(),
    "L"=>c.SemanticList(),"LI"=>c.SemanticListItem(),"Lbl"=>c.SemanticListLabel(),"LBody"=>c.SemanticListItemBody(),"Link"=>c.SemanticLink(S(n,"alt")),
    "Figure"=>c.SemanticFigure(S(n,"alt")),"Formula"=>c.SemanticFormula(S(n,"alt")),"NonStruct"=>c.SemanticLanguage(S(n,"language")),"Table"=>c.SemanticTable(),"Ignore"=>c.SemanticIgnore(),
    _=>throw new ArgumentException("Unknown semantic fixture role")
   };Compose(n.GetProperty("child"),container);break;
  }
  case "section_link":Compose(n.GetProperty("child"),c.SectionLink(S(n,"name")));break;
  case "hyperlink":Compose(n.GetProperty("child"),c.Hyperlink(S(n,"url")));break;
  case "height":Compose(n.GetProperty("child"),c.Height(F(n,"value")));break;
  case "min_height":Compose(n.GetProperty("child"),c.MinHeight(F(n,"value")));break;
  case "align_center":Compose(n.GetProperty("child"),c.AlignCenter());break;
  case "align_right":Compose(n.GetProperty("child"),c.AlignRight());break;
  case "align_middle":Compose(n.GetProperty("child"),c.AlignMiddle());break;
  case "background":Compose(n.GetProperty("child"),c.Background(S(n,"value")));break;
  case "border":Compose(n.GetProperty("child"),c.Border(F(n,"value",1)).BorderColor("#000000"));break;
  case "styled_box":{
   var box=c;
   if(n.TryGetProperty("border",out var border))box=box.BorderLeft(border[0].GetSingle()).BorderTop(border[1].GetSingle()).BorderRight(border[2].GetSingle()).BorderBottom(border[3].GetSingle()).BorderColor("#224466");
   if(n.TryGetProperty("radius",out var radius))box=box.CornerRadiusTopLeft(radius[0].GetSingle()).CornerRadiusTopRight(radius[1].GetSingle()).CornerRadiusBottomRight(radius[2].GetSingle()).CornerRadiusBottomLeft(radius[3].GetSingle());
   switch(S(n,"border_alignment")){case "inside":box=box.BorderAlignmentInside();break;case "middle":box=box.BorderAlignmentMiddle();break;case "outside":box=box.BorderAlignmentOutside();break;}
   box=n.TryGetProperty("gradient",out var angle)?box.BackgroundLinearGradient(angle.GetSingle(),new Color[]{"#99aaff","#ffdd66","#aa3388"}):box.Background("#99bbdd");
   if(n.TryGetProperty("border_gradient",out var bangle))box=box.BorderLinearGradient(bangle.GetSingle(),new Color[]{"#33aa66","#dd3333"});
   if(n.TryGetProperty("shadow",out var shadow)&&shadow.GetBoolean())box=box.Shadow(new BoxShadowStyle{OffsetX=4,OffsetY=6,Spread=2,Blur=3,Color="#88775544"});
   Compose(n.GetProperty("child"),box);break;
  }
  case "stop_paging":Compose(n.GetProperty("child"),c.StopPaging());break;
  case "show_bool":Compose(n.GetProperty("child"),c.ShowIf(n.GetProperty("value").GetBoolean()));break;
  case "border_edges":{
   var target=S(n,"edge") switch {"horizontal"=>c.BorderHorizontal(2),"vertical"=>c.BorderVertical(2),"before_transparent"=>c.BorderColor("#00000000").Border(2),"after_transparent"=>c.Border(2).BorderColor("#00000000"),_=>c.Border(2)};
   Compose(n.GetProperty("child"),target);break;
  }
  case "component_border":c.Border(1).Component(new BorderComponent());break;
  case "show_entire":Compose(n.GetProperty("child"),c.ShowEntire());break;
  case "ensure_space":Compose(n.GetProperty("child"),c.EnsureSpace(F(n,"value")));break;
  case "prevent_page_break":Compose(n.GetProperty("child"),c.PreventPageBreak());break;
  case "show_once":Compose(n.GetProperty("child"),c.ShowOnce());break;
  case "skip_once":Compose(n.GetProperty("child"),c.SkipOnce());break;
  case "repeat":Compose(n.GetProperty("child"),c.Repeat());break;
  case "page_break":c.PageBreak();break;
  case "scale_to_fit":Compose(n.GetProperty("child"),c.ScaleToFit());break;
  case "scale":Compose(n.GetProperty("child"),c.Scale(F(n,"value")));break;
  case "scale_xy":Compose(n.GetProperty("child"),c.ScaleHorizontal(F(n,"x",1)).ScaleVertical(F(n,"y",1)));break;
  case "rotate":Compose(n.GetProperty("child"),c.Rotate(F(n,"value")));break;
  case "shrink":Compose(n.GetProperty("child"),c.Shrink());break;
  case "unconstrained":Compose(n.GetProperty("child"),c.Unconstrained());break;
  case "debug_area":Compose(n.GetProperty("child"),c.DebugArea(S(n,"text")));break;
  case "show_predicate":Compose(n.GetProperty("child"),c.ShowIf(ctx=>ctx.PageNumber%2==1));break;
  case "lazy_cached":c.LazyWithCache(child=>Compose(n.GetProperty("child"),child));break;

  case "rotate_left":Compose(n.GetProperty("child"),c.RotateLeft());break;
  case "rotate_right":Compose(n.GetProperty("child"),c.RotateRight());break;
  case "line":{var line=n.TryGetProperty("vertical",out var vertical)&&vertical.GetBoolean()?c.LineVertical(F(n,"value",1)):c.LineHorizontal(F(n,"value",1));if(n.TryGetProperty("dash",out var dash))line.LineDashPattern(dash.EnumerateArray().Select(x=>x.GetSingle()).ToArray());if(n.TryGetProperty("gradient",out var gradient)&&gradient.GetBoolean())line.LineGradient(new Color[]{"#ff3344","#33aa66"});break;}
  case "placeholder":c.Placeholder(S(n,"text"));break;
  case "svg":c.Svg(S(n,"text")).FitArea();break;
  case "decoration":c.Decoration(d=>{Compose(n.GetProperty("before"),d.Before());Compose(n.GetProperty("content"),d.Content());Compose(n.GetProperty("after"),d.After());});break;
  case "layers":c.Layers(d=>{foreach(var item in n.GetProperty("items").EnumerateArray())Compose(item.GetProperty("content"),item.GetProperty("primary").GetBoolean()?d.PrimaryLayer():d.Layer());});break;
  case "multi_column":c.MultiColumn(d=>{d.Columns((int)F(n,"count",2));d.Spacing(F(n,"spacing"));d.BalanceHeight(n.GetProperty("balance").GetBoolean());Compose(n.GetProperty("content"),d.Content());Compose(n.GetProperty("spacer"),d.Spacer());});break;
  case "table":c.Table(d=>{
   if(n.TryGetProperty("extend_last",out var extend)&&extend.GetBoolean())d.ExtendLastCellsToTableBottom();
   d.ColumnsDefinition(cols=>{foreach(var col in n.GetProperty("columns").EnumerateArray())cols.RelativeColumn(col.GetSingle());});
   if(n.TryGetProperty("header",out var head))d.Header(h=>{foreach(var cell in head.EnumerateArray())Compose(cell,h.Cell());});
   if(n.TryGetProperty("footer",out var foot))d.Footer(h=>{foreach(var cell in foot.EnumerateArray())Compose(cell,h.Cell());});
   if(n.TryGetProperty("header_cells",out var headerCells))d.Header(h=>{foreach(var item in headerCells.EnumerateArray()){
    ITableCellContainer cell=h.Cell();
    if(item.TryGetProperty("column",out var column))cell=cell.Column(column.GetUInt32());
    if(item.TryGetProperty("row",out var row))cell=cell.Row(row.GetUInt32());
    if(item.TryGetProperty("column_span",out var columnSpan))cell=cell.ColumnSpan(columnSpan.GetUInt32());
    if(item.TryGetProperty("row_span",out var rowSpan))cell=cell.RowSpan(rowSpan.GetUInt32());
    if(item.TryGetProperty("semantic_header",out var semanticHeader)&&semanticHeader.GetBoolean())cell=cell.SemanticHorizontalHeader();
    Compose(item.GetProperty("content"),cell);
   }});
   foreach(var item in n.GetProperty("cells").EnumerateArray()){
    ITableCellContainer cell=d.Cell();
    if(item.TryGetProperty("column",out var column))cell=cell.Column(column.GetUInt32());
    if(item.TryGetProperty("row",out var row))cell=cell.Row(row.GetUInt32());
    if(item.TryGetProperty("column_span",out var columnSpan))cell=cell.ColumnSpan(columnSpan.GetUInt32());
    if(item.TryGetProperty("row_span",out var rowSpan))cell=cell.RowSpan(rowSpan.GetUInt32());
    if(item.TryGetProperty("semantic_header",out var semanticHeader)&&semanticHeader.GetBoolean())cell=cell.SemanticHorizontalHeader();
    Compose(item.GetProperty("content"),cell);
   }
  });break;
  default:throw new ArgumentException("Unknown fixture node "+kind);
 }
}
sealed class DynamicRows(int count):IDynamicComponent<int> {
 public int State {get;set;}
 public DynamicComponentComposeResult Compose(QuestPDF.Elements.DynamicContext context){
  var total=context.TotalPages==int.MaxValue?"?":context.TotalPages.ToString();
  var header=context.CreateElement(c=>c.Text($"Dynamic page {context.PageNumber} / {total}").Bold());
  var height=header.Size.Height;var rows=new List<QuestPDF.Elements.IDynamicElement>();
  for(var index=State;index<count;index++){
   var row=context.CreateElement(c=>c.Width(context.AvailableSize.Width).Text($"Row {index:000}: dynamically measured content keeps its state."));
   if(height+3+row.Size.Height>context.AvailableSize.Height+0.01f)break;
   height+=3+row.Size.Height;rows.Add(row);
  }
  State+=rows.Count;
  var content=context.CreateElement(c=>c.Column(col=>{col.Spacing(3);col.Item().Element(header);foreach(var row in rows)col.Item().Element(row);}));
  return new DynamicComponentComposeResult {Content=content,HasMoreContent=State<count};
 }
}
sealed class PositionProbe(string id):IDynamicComponent {
 public DynamicComponentComposeResult Compose(QuestPDF.Elements.DynamicContext context){
  var positions=context.GetContentCapturedPositions(id);
  var lines=positions.Select(p=>FormattableString.Invariant($"Page {p.PageNumber}: X={p.X:F3}, Y={p.Y:F3}, W={p.Width:F3}, H={p.Height:F3}"));
  var content=context.CreateElement(c=>c.Text($"Captured {positions.Count}\n"+string.Join("\n",lines)).FontSize(8));
  return new DynamicComponentComposeResult{Content=content,HasMoreContent=false};
 }
}

sealed class BorderComponent:IComponent {
 public void Compose(IContainer c){c.Border(2).Padding(4).Text("Component boundary");}
}
