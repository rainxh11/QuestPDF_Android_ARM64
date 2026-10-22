using QuestPdf.Android.Native;

var directory = args.Length == 0 ? Path.GetFullPath("artifacts/host-features") : Path.GetFullPath(args[0]);
var (report, success) = FeatureSuite.RunAll(directory);
Console.WriteLine(report);
return success ? 0 : 1;
