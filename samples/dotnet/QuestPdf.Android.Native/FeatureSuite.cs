using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using QuestPDF.Drawing;
using QuestPDF.Drawing.Exceptions;
using QuestPDF.Elements;
using QuestPDF.Fluent;
using QuestPDF.Helpers;
using QuestPDF.Infrastructure;

namespace QuestPdf.Android.Native;

public sealed record FeatureResult(int Id, string Name, bool Passed, long Milliseconds, int Bytes, string? Error);
public sealed record SuiteReport(string Runtime, string Architecture, List<FeatureResult> Results);

[JsonSerializable(typeof(SuiteReport))]
internal partial class ReportJsonContext : JsonSerializerContext;

public static class FeatureSuite
{
    public static readonly string[] Names = [
        "Overview and metadata", "Multipage table and pagination", "Arabic RTL and rich text",
        "PNG JPEG WebP images", "SVG paths and text", "Styled boxes layers transforms",
        "Links and destinations", "Dynamic and lazy content", "PDF/A-3b generation",
        "Tagged PDF/UA generation", "Merge and select pages", "Overlay and underlay",
        "Unicode attachment", "AES-128 encryption roundtrip", "AES-256 encryption roundtrip",
        "PNG JPEG WebP raster exports", "SVG export", "Expected layout failure",
        "Repeated generation", "Concurrent generation", "XMP metadata extension", "System fonts"
    ];
    private static readonly object InitLock = new();
    private static bool initialized;

    private static void Initialize(string directory)
    {
        Directory.CreateDirectory(directory);
        lock (InitLock)
        {
            if (initialized) return;
            QuestPDF.Settings.License = LicenseType.Community;
            QuestPDF.Settings.FontDiscoveryPath = null;
            QuestPDF.Settings.TemporaryStoragePath = directory;
            foreach (var name in new[] { "assets.NotoSansArabic.ttf" })
            {
                using var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(name)
                    ?? throw new InvalidOperationException($"Missing embedded font: {name}");
                FontManager.RegisterFontFromStream(stream);
            }
            initialized = true;
        }
    }

    private static Document Page(string title, Action<IContainer> content) => Document.Create(doc =>
    {
        doc.Page(page =>
        {
            page.Size(PageSizes.A4);
            page.Margin(36);
            page.DefaultTextStyle(x => x.FontFamily("Lato").FontSize(11));
            page.Header().PaddingBottom(14).Text(title).FontSize(22).Bold().FontColor("#174E68");
            page.Content().Element(content);
            page.Footer().PaddingTop(12).AlignCenter().Text(text =>
            {
                text.Span("QuestPDF Android  |  ");
                text.CurrentPageNumber();
                text.Span(" / ");
                text.TotalPages();
            });
        });
    }).WithMetadata(new DocumentMetadata { Title = title, Author = "QuestPDF Android test suite", Language = "en-US" });

    private static byte[] Asset(string name)
    {
        using var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("assets." + name)
            ?? throw new FileNotFoundException(name);
        using var output = new MemoryStream();
        stream.CopyTo(output);
        return output.ToArray();
    }

    public static byte[] Generate(int id, string directory)
    {
        if (id < 0 || id >= Names.Length) throw new ArgumentOutOfRangeException(nameof(id));
        Initialize(directory);
        var work = Path.Combine(directory, "fixture-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(work);
        try { return GenerateCore(id, work); }
        finally { Directory.Delete(work, true); }
    }

    private static byte[] GenerateCore(int id, string work)
    {
        var title = Names[id];
        switch (id)
        {
            case 0:
                return Page(title, c => c.Column(col =>
                {
                    col.Spacing(16);
                    col.Item().Text("Generated entirely on Android by the local QuestPDF source, compiled with .NET NativeAOT.");
                    col.Item().Background("#EAF3F5").CornerRadius(10).Padding(18).Text("Flutter -> Dart FFI -> .NET NativeAOT -> QuestPDF -> native Skia / qpdf -> PDF bytes -> pdfrx");
                    col.Item().Text($"Runtime: {RuntimeInformation.RuntimeIdentifier}\nArchitecture: {RuntimeInformation.ProcessArchitecture}");
                    foreach (var name in Names) col.Item().Text(name);
                })).GeneratePdf();
            case 1:
                return Page(title, c => c.Table(table =>
                {
                    table.ColumnsDefinition(cols => { cols.ConstantColumn(45); cols.RelativeColumn(3); cols.RelativeColumn(); });
                    table.Header(header => { foreach (var text in new[] { "Row", "Description", "Amount" }) header.Cell().Background("#174E68").Padding(7).Text(text).FontColor("#FFFFFF"); });
                    for (var row = 1; row <= 160; row++)
                    {
                        table.Cell().BorderBottom(0.5f).Padding(7).Text(row.ToString());
                        table.Cell().BorderBottom(0.5f).Padding(7).Text($"Pagination fixture item {row:000}");
                        table.Cell().BorderBottom(0.5f).Padding(7).AlignRight().Text($"{row * 12.5:0.00}");
                    }
                })).GeneratePdf();
            case 2:
                return Page(title, c => c.Column(col =>
                {
                    col.Spacing(24);
                    col.Item().Text(t => { t.Span("Rich text: "); t.Span("bold, ").Bold(); t.Span("underlined, ").Underline(); t.Span("coloured").FontColor("#B42E46"); });
                    col.Item().ContentFromRightToLeft().Text("مرحبا بالعالم - إنشاء مستند PDF على أندرويد").FontFamily("Noto Sans Arabic").FontSize(22);
                    col.Item().ContentFromRightToLeft().Text("اختبار العربية مع الأرقام 12345 و Flutter و QuestPDF").FontFamily("Noto Sans Arabic").FontSize(18);
                    col.Item().Text("Accents: français, élève, déjà, Ångström, Ελληνικά, zażółć.");
                })).GeneratePdf();
            case 3:
                return Page(title, c => c.Column(col =>
                {
                    col.Spacing(12);
                    foreach (var ext in new[] { "png", "jpg", "webp" })
                    {
                        col.Item().Text(ext.ToUpperInvariant());
                        col.Item().Height(150).Image(Asset("sample." + ext)).FitArea();
                    }
                })).GeneratePdf();
            case 4:
                return Page(title, c => c.Height(260).Svg("""
                    <svg xmlns="http://www.w3.org/2000/svg" width="480" height="250">
                    <defs><linearGradient id="g"><stop stop-color="#174E68"/><stop offset="1" stop-color="#37B7A5"/></linearGradient></defs>
                    <rect x="5" y="5" width="470" height="240" rx="22" fill="url(#g)"/>
                    <path d="M50 160 Q150 20 240 140 T430 100" fill="none" stroke="white" stroke-width="8"/>
                    <text x="40" y="210" fill="white" font-family="Lato" font-size="26">SVG text and paths</text></svg>
                    """)).GeneratePdf();
            case 5:
                return Page(title, c => c.Column(col =>
                {
                    col.Spacing(35);
                    col.Item().Border(2).BorderColor("#37B7A5").CornerRadius(18).Background("#EAF3F5")
                        .Shadow(new BoxShadowStyle { Color = "#808080", Blur = 8, OffsetX = 3, OffsetY = 3 }).Padding(24).Text("Rounded border and shadow");
                    col.Item().Height(150).Layers(layers =>
                    {
                        layers.Layer().Background("#EAF3F5");
                        layers.PrimaryLayer().AlignCenter().AlignMiddle().Rotate(12).Text("Layers + rotation + scaling").FontSize(20);
                    });
                    col.Item().Scale(0.8f).Text("Scaled content");
                })).GeneratePdf();
            case 6:
                return Page(title, c => c.Column(col =>
                {
                    col.Spacing(20);
                    col.Item().Hyperlink("https://www.questpdf.com/?test=é").Text("External UTF-8 URL").Underline();
                    col.Item().SectionLink("destination-é").Text("Jump to destination").Underline();
                    col.Item().PageBreak();
                    col.Item().Section("destination-é").Text("Internal destination reached").FontSize(24);
                })).GeneratePdf();
            case 7:
                return Page(title, c => c.Column(col =>
                {
                    col.Item().Dynamic(new PageAwareComponent());
                    col.Item().Lazy(x => x.Text("Lazy content created during rendering"));
                })).GeneratePdf();
            case 8:
                return Page(title, c => c.Text("Archival PDF/A-3b generation fixture"))
                    .WithSettings(new DocumentSettings { PDFA_Conformance = PDFA_Conformance.PDFA_3B }).GeneratePdf();
            case 9:
                return Page(title, c => c.Column(col =>
                {
                    col.Item().Text("Accessible tagged content");
                    col.Item().Text("Inspect StructTreeRoot and language; use veraPDF for formal compliance.");
                })).WithSettings(new DocumentSettings { PDFUA_Conformance = PDFUA_Conformance.PDFUA_1 }).GeneratePdf();
            case >= 10 and <= 14:
            case 20:
                return Operations(id, work);
            case 15:
            {
                var document = Page(title, c => c.Text("Raster export test"));
                foreach (var format in new[] { ImageFormat.Png, ImageFormat.Jpeg, ImageFormat.Webp })
                {
                    var bytes = document.GenerateImages(new ImageGenerationSettings { ImageFormat = format, RasterDpi = 72 }).Single();
                    if (bytes.Length < 100) throw new Exception($"Empty {format} export");
                    File.WriteAllBytes(Path.Combine(Path.GetDirectoryName(work)!, format + ".image"), bytes);
                }
                return document.GeneratePdf();
            }
            case 16:
            {
                var document = Page(title, c => c.Text("SVG export fixture"));
                var svg = document.GenerateSvg().Single();
                if (!svg.Contains("<svg")) throw new Exception("SVG export missing root");
                File.WriteAllText(Path.Combine(Path.GetDirectoryName(work)!, "export.svg"), svg);
                return document.GeneratePdf();
            }
            case 17:
                try { Page(title, c => c.Width(10).Height(10).Text(new string('X', 200))).GeneratePdf(); }
                catch (DocumentLayoutException) { return Page(title, c => c.Text("PASS: layout exception caught safely across the native boundary.")).GeneratePdf(); }
                throw new Exception("Expected DocumentLayoutException was not thrown");
            case 18:
                for (var i = 0; i < 20; i++) CheckPdf(Page(title, c => c.Text($"Iteration {i}")).GeneratePdf());
                return Page(title, c => c.Text("20 repeated documents generated successfully.")).GeneratePdf();
            case 19:
                Parallel.For(0, 8, i => CheckPdf(Page(title, c => c.Text($"Concurrent worker {i}")).GeneratePdf()));
                return Page(title, c => c.Text("8 concurrent documents generated successfully.")).GeneratePdf();
            case 21:
            {
                var fonts = FontManager.GetSystemFonts();
                if (fonts.Count == 0) throw new Exception("Android system font discovery returned no fonts");
                var font = fonts.FirstOrDefault(x => x.FamilyName is "Roboto" or "Arial") ?? fonts.First();
                QuestPDF.Settings.UseSystemFonts = true;
                try
                {
                    return Page(title, c => c.Text($"System font: {font.FamilyName}. Discovered {fonts.Count} typefaces.")
                        .FontFamily(font.FamilyName).FontSize(18)).GeneratePdf();
                }
                finally { QuestPDF.Settings.UseSystemFonts = false; }
            }
            default: throw new ArgumentOutOfRangeException(nameof(id));
        }
    }

    private static byte[] Operations(int id, string work)
    {
        var input = Path.Combine(work, "entrée-é.pdf");
        var output = Path.Combine(work, "résultat.pdf");
        var second = Path.Combine(work, "second.pdf");
        Page("SOURCE", c => c.Column(col => { col.Item().Text("First page"); col.Item().PageBreak(); col.Item().Text("Second page"); }))
            .WithSettings(new DocumentSettings { PDFA_Conformance = PDFA_Conformance.PDFA_3B }).GeneratePdf(input);
        Page("WATERMARK", c => c.AlignBottom().Text("Overlay watermark")).GeneratePdf(second);
        var operation = DocumentOperation.LoadFile(input);
        switch (id)
        {
            case 10: operation.TakePages("2").MergeFile(second); break;
            case 11: operation.OverlayFile(new() { FilePath = second }).UnderlayFile(new() { FilePath = second }); break;
            case 12:
                var attachment = Path.Combine(work, "pièce-é.txt");
                File.WriteAllText(attachment, "Android ARM64 UTF-8 attachment: مرحبا");
                operation.AddAttachment(new() { FilePath = attachment, AttachmentName = "pièce-é.txt" }); break;
            case 13: operation.Encrypt(new DocumentOperation.Encryption128Bit { UserPassword = "user-é", OwnerPassword = "owner-é" }); break;
            case 14: operation.Encrypt(new DocumentOperation.Encryption256Bit { UserPassword = "user-é", OwnerPassword = "owner-é" }); break;
            case 20: operation.ExtendMetadata("<rdf:Description xmlns:android=\"urn:questpdf:android\" android:Test=\"ARM64\"/>"); break;
        }
        operation.Save(output);
        if (id is 13 or 14)
        {
            File.Copy(output, Path.Combine(Path.GetDirectoryName(work)!, $"encrypted-{id}.pdf"), true);
            var decrypted = Path.Combine(work, "decrypted.pdf");
            DocumentOperation.LoadFile(output, "user-é").Decrypt().Save(decrypted);
            return File.ReadAllBytes(decrypted);
        }
        return File.ReadAllBytes(output);
    }

    private sealed class PageAwareComponent : IDynamicComponent
    {
        public DynamicComponentComposeResult Compose(DynamicContext context) => new()
        {
            Content = context.CreateElement(c => c.Text($"Dynamic content on page {context.PageNumber}")),
            HasMoreContent = false
        };
    }

    private static void CheckPdf(byte[] bytes)
    {
        if (bytes.Length < 100 || !bytes.AsSpan(0, 5).SequenceEqual("%PDF-"u8))
            throw new InvalidDataException("Invalid PDF output");
    }

    public static (string Report, bool Success) RunAll(string directory)
    {
        Initialize(directory);
        var results = new List<FeatureResult>();
        for (var id = 0; id < Names.Length; id++)
        {
            var timer = Stopwatch.StartNew();
            try
            {
                var bytes = Generate(id, directory);
                CheckPdf(bytes);
                File.WriteAllBytes(Path.Combine(directory, $"feature-{id:00}.pdf"), bytes);
                results.Add(new(id, Names[id], true, timer.ElapsedMilliseconds, bytes.Length, null));
            }
            catch (Exception ex) { results.Add(new(id, Names[id], false, timer.ElapsedMilliseconds, 0, ex.ToString())); }
        }
        var report = JsonSerializer.Serialize(new SuiteReport(RuntimeInformation.RuntimeIdentifier,
            RuntimeInformation.ProcessArchitecture.ToString(), results), ReportJsonContext.Default.SuiteReport);
        File.WriteAllText(Path.Combine(directory, "results.json"), report);
        return (report, results.All(x => x.Passed));
    }
}
