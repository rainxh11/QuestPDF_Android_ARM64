import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:questpdf_android_demo/status_banner.dart';

void main() {
  testWidgets('shows native result and busy state', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: StatusBanner(message: 'Generating...', busy: true)));
    expect(find.text('Generating...'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
  });
}
