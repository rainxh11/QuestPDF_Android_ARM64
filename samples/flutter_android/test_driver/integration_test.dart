import 'dart:convert';
import 'dart:io';
import 'package:integration_test/integration_test_driver.dart';

Future<void> main() => integrationDriver(responseDataCallback: (data) async {
  if (data == null) return;
  final directory = Directory('../../artifacts/emulator');
  await directory.create(recursive: true);
  for (final screenshot in (data['screenshots'] as List<dynamic>? ?? [])) {
    await File('${directory.path}/${screenshot['screenshotName']}.png')
        .writeAsBytes((screenshot['bytes'] as List<dynamic>).cast<int>());
  }
  final summary = Map<String, dynamic>.from(data)..remove('screenshots');
  final runtime = (summary['nativeReport'] as Map<String, dynamic>?)?['Runtime'] ?? 'unknown';
  await File('${directory.path}/$runtime-integration-results.json')
      .writeAsString(const JsonEncoder.withIndent('  ').convert(summary));
});
