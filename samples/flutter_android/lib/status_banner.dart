import 'package:flutter/material.dart';
import 'package:flutter/widget_previews.dart';

class StatusBanner extends StatelessWidget {
  const StatusBanner({super.key, required this.message, this.busy = false});
  final String message;
  final bool busy;
  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity, padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
    color: Theme.of(context).colorScheme.surfaceContainer,
    child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      if (busy) const Padding(padding: EdgeInsets.only(bottom: 8), child: LinearProgressIndicator()),
      Text(message, key: const ValueKey('native-status'), maxLines: 5, overflow: TextOverflow.ellipsis),
    ]),
  );
}

@Preview(name: 'Native feature status', size: Size(390, 90))
Widget statusBannerPreview() => const MaterialApp(home: Scaffold(
  body: StatusBanner(message: '22/22 native features passed | Arm64'),
));
