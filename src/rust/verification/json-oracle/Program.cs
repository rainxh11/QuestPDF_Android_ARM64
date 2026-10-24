using System.Text.Json;
using QuestPDF;
using QuestPDF.Fluent;
using QuestPDF.Infrastructure;
using RisDocumentServer.Documents;
using RisDocumentServer.Models;
Settings.License=LicenseType.Community;
foreach(var file in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory,"Assets"),"*.TTF"))QuestPDF.Drawing.FontManager.RegisterFontFromFile(file);
Directory.CreateDirectory(args[1]);
var options=new JsonSerializerOptions{PropertyNameCaseInsensitive=true};
foreach(var test in JsonDocument.Parse(File.ReadAllText(args[0])).RootElement.EnumerateArray()){
 var payload=test.GetProperty("payload");var kind=test.GetProperty("kind").GetString();
 IDocument document=kind switch {
  "receipt" or "debt-receipt"=>new InvoiceReceiptDocument(payload.Deserialize<InvoiceReceiptModel>(options)!),
  "large-receipt"=>new LargeReceiptDocument(payload.Deserialize<LargeReceiptModel>(options)!),
  "debt-report"=>new DebtDocument(payload.Deserialize<RisDebtReport>(options)!),
  "debt-out-report"=>new DebtOutReportByUsers(payload.Deserialize<RisDebtOutReport>(options)!),
  "income-report"=>new IncomeReportByUsers(payload.Deserialize<RisIncomeReport>(options)!),
  "substitute-report"=>new SubstituteDoctorMonthlyReport(payload.Deserialize<SubstituteDoctorReportModel>(options)!),
  _=>throw new ArgumentException(kind)
 };
 document.GeneratePdf(Path.Combine(args[1],test.GetProperty("name").GetString()+".pdf"));
}
static class Frozen {
 public static DateTime Now=>new(2026,10,24,12,0,0,DateTimeKind.Local);
 public static DateTimeOffset Offset=>new(Now);
}
