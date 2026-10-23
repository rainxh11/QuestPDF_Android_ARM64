import 'dart:convert';
import 'dart:io';
import 'dart:ffi';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:path_provider/path_provider.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:questpdf_android_demo/main.dart';
import 'package:questpdf_android_demo/native_bridge.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Future<void> waitFor(WidgetTester tester, bool Function() ready) async {
    final deadline = DateTime.now().add(const Duration(minutes: 5));
    // Apply state changes from the preceding tap before inspecting the tree.
    await tester.pump();
    while (!ready()) {
      if (DateTime.now().isAfter(deadline)) fail('Timed out waiting for app: ${find.byKey(const ValueKey('native-status')).evaluate().map((e) => (e.widget as Text).data)}');
      await tester.pump(const Duration(milliseconds: 300));
    }
  }

  testWidgets('Android NativeAOT generation, essential features and pdfrx preview', (tester) async {
    await tester.pumpWidget(const QuestPdfApp());
    await waitFor(tester, () => find.byKey(const ValueKey('preview-ready')).evaluate().isNotEmpty);
    expect(find.byType(PdfViewer), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('run-tests')));
    await waitFor(tester, () => find.textContaining('native features passed').evaluate().isNotEmpty);

    final cache = await getApplicationCacheDirectory();
    final directory = '${cache.path}/questpdf-tests';
    final report = jsonDecode(await File('$directory/results.json').readAsString()) as Map<String, dynamic>;
    final (architecture, cpu) = switch (Abi.current()) {
      Abi.androidArm => ('Arm', 'arm'),
      Abi.androidArm64 => ('Arm64', 'arm64'),
      Abi.androidX64 => ('X64', 'x64'),
      final abi => throw UnsupportedError('Unsupported test ABI: $abi'),
    };
    expect(report['Architecture'], architecture);
    expect(report['Runtime'], 'linux-bionic-$cpu');
    expect(report['PointerSize'], sizeOf<IntPtr>());
    final results = report['Results'] as List<dynamic>;
    expect(results.length, featureNames.length);
    for (final result in results) {
      expect(result['Passed'], true, reason: '${result['Name']}: ${result['Error']}');
    }

    var renderedPages = 0;
    var tablePageCount = 0;
    for (var id = 0; id < featureNames.length; id++) {
      final document = await PdfDocument.openFile('$directory/feature-${id.toString().padLeft(2, '0')}.pdf');
      try {
        expect(document.pages, isNotEmpty);
        if (id == 1) {
          expect(document.pages.length, greaterThan(3));
          tablePageCount = document.pages.length;
        }
        if (id == 6 || id == 10) expect(document.pages.length, 2);
        for (final page in document.pages) {
          final image = await page.render(fullWidth: 300, fullHeight: page.height * 300 / page.width);
          expect(image, isNotNull, reason: 'PDFium failed to render feature $id page ${page.pageNumber}');
          expect(image!.pixels.any((pixel) => pixel < 180), true, reason: 'Blank page for feature $id');
          image.dispose();
          final text = await page.loadText();
          expect(text?.fullText.trim(), isNotEmpty, reason: 'Missing text for feature $id');
          renderedPages++;
        }
        if (id == 6) expect((await document.pages.first.loadLinks()).length, greaterThanOrEqualTo(2));
      } finally {
        await document.dispose();
      }
    }

    final invalid = await generateDocument(999, directory);
    expect(invalid.status, 1);
    expect(invalid.text, contains('ArgumentOutOfRangeException'));
    final recovered = await generateDocument(0, directory);
    expect(recovered.status, 0, reason: 'Native bridge must recover after managed errors');

    await tester.tap(find.byKey(const ValueKey('feature-selector')));
    await tester.pumpAndSettle();
    await tester.tap(find.text(featureNames[1]).last);
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('generate')));
    await waitFor(tester, () => find.text('PDF preview ready · $tablePageCount pages').evaluate().isNotEmpty);
    final previewController = tester.widget<PdfViewer>(find.byType(PdfViewer)).controller!;
    await tester.tap(find.byKey(const ValueKey('next-page')));
    await waitFor(tester, () => previewController.pageNumber == 2);
    await tester.tap(find.byKey(const ValueKey('previous-page')));
    await waitFor(tester, () => previewController.pageNumber == 1);
    await binding.convertFlutterSurfaceToImage();
    await tester.pumpAndSettle();
    await binding.takeScreenshot('questpdf-$cpu-preview');
    binding.reportData ??= <String, dynamic>{};
    binding.reportData!['nativeReport'] = report;
    binding.reportData!['renderedPages'] = renderedPages;
  }, timeout: const Timeout(Duration(minutes: 20)));
}
