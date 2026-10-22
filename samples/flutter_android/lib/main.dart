import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'package:pdfrx/pdfrx.dart';
import 'native_bridge.dart';
import 'status_banner.dart';

void main() => runApp(const QuestPdfApp());

class QuestPdfApp extends StatelessWidget {
  const QuestPdfApp({super.key});
  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'QuestPDF Android',
    theme: ThemeData(colorSchemeSeed: const Color(0xff174e68), useMaterial3: true),
    home: const DocumentScreen(),
  );
}

class DocumentScreen extends StatefulWidget {
  const DocumentScreen({super.key});
  @override
  State<DocumentScreen> createState() => _DocumentScreenState();
}

class _DocumentScreenState extends State<DocumentScreen> {
  final controller = PdfViewerController();
  Uint8List? bytes;
  String status = 'Preparing native document generator...';
  String? report;
  int feature = 0;
  int generation = 0;
  bool busy = false;
  bool viewerReady = false;

  @override
  void initState() { super.initState(); generate(); }

  Future<String> directory() async {
    final cache = await getApplicationCacheDirectory();
    final path = Directory('${cache.path}/questpdf-tests');
    await path.create(recursive: true);
    return path.path;
  }

  Future<void> generate() async {
    setState(() { busy = true; viewerReady = false; status = 'Generating ${featureNames[feature]}...'; });
    try {
      final result = await generateDocument(feature, await directory());
      if (!mounted) return;
      if (result.status != 0) throw StateError(result.text);
      setState(() { bytes = result.bytes; generation++; status = '${result.bytes.length} PDF bytes from .NET NativeAOT'; });
    } catch (error) {
      if (mounted) setState(() { status = error.toString(); });
    } finally {
      if (mounted) setState(() { busy = false; });
    }
  }

  Future<void> runTests() async {
    setState(() { busy = true; status = 'Running ${featureNames.length} native feature fixtures...'; });
    try {
      final result = await runFeatureSuite(await directory());
      final json = jsonDecode(result.text) as Map<String, dynamic>;
      final results = json['Results'] as List<dynamic>;
      final passed = results.where((r) => r['Passed'] == true).length;
      if (!mounted) return;
      setState(() {
        report = const JsonEncoder.withIndent('  ').convert(json);
        status = '$passed/${results.length} native features passed | ${json['Architecture']}';
      });
    } catch (error) {
      if (mounted) setState(() { status = error.toString(); });
    } finally {
      if (mounted) setState(() { busy = false; });
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('QuestPDF · Android NativeAOT')),
    body: SafeArea(child: Column(children: [
      Padding(padding: const EdgeInsets.symmetric(horizontal: 16), child: DropdownButtonFormField<int>(
        key: const ValueKey('feature-selector'), initialValue: feature, isExpanded: true,
        decoration: const InputDecoration(labelText: 'Feature document'),
        items: [for (var i = 0; i < featureNames.length; i++)
          DropdownMenuItem(value: i, child: Text(featureNames[i], overflow: TextOverflow.ellipsis))],
        onChanged: busy ? null : (value) => setState(() { feature = value!; }),
      )),
      Padding(padding: const EdgeInsets.all(12), child: Wrap(spacing: 8, runSpacing: 8, children: [
        FilledButton.icon(key: const ValueKey('generate'), onPressed: busy ? null : generate,
          icon: const Icon(Icons.description_outlined), label: const Text('Generate PDF')),
        OutlinedButton(key: const ValueKey('run-tests'), onPressed: busy ? null : runTests, child: const Text('Run all tests')),
        if (report != null) IconButton(tooltip: 'Test results', icon: const Icon(Icons.fact_check),
          onPressed: () => showDialog<void>(context: context, builder: (c) => AlertDialog(
            title: const Text('Native feature results'), content: SingleChildScrollView(child: SelectableText(report!)),
            actions: [TextButton(onPressed: () => Navigator.pop(c), child: const Text('Close'))],
          ))),
      ])),
      StatusBanner(message: status, busy: busy),
      if (viewerReady) Row(mainAxisAlignment: MainAxisAlignment.center, children: [
        IconButton(key: const ValueKey('previous-page'), tooltip: 'Previous page',
          onPressed: () => controller.goToPage(pageNumber: ((controller.pageNumber ?? 1) - 1).clamp(1, controller.pageCount)),
          icon: const Icon(Icons.chevron_left)),
        Text('PDF preview ready · ${controller.pageCount} pages', key: const ValueKey('preview-ready')),
        IconButton(key: const ValueKey('next-page'), tooltip: 'Next page',
          onPressed: () => controller.goToPage(pageNumber: ((controller.pageNumber ?? 1) + 1).clamp(1, controller.pageCount)),
          icon: const Icon(Icons.chevron_right)),
      ]),
      Expanded(child: bytes == null ? const Center(child: Text('Generate a document to preview it here.'))
        : PdfViewer.data(bytes!, key: ValueKey('pdf-$generation'), sourceName: 'questpdf-$generation.pdf', controller: controller,
            params: PdfViewerParams(onViewerReady: (document, ctrl) {
              if (mounted) setState(() { viewerReady = true; });
            }))),
    ])),
  );
}
