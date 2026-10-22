# QuestPDF Android demo

The app calls the local QuestPDF C# source compiled as a .NET NativeAOT shared
library, receives PDF bytes through Dart FFI, and displays them with pdfrx. Generation
runs in a background Dart isolate. It supports `arm64-v8a` and `x86_64`.

Build the native libraries and bridge first, as described in
[the port overview and build guide](../../docs/android-arm64.md).

```powershell
flutter pub get
flutter build apk --debug --target-platform android-arm64,android-x64
```

The final demo is built with `--profile` so both Dart and .NET use AOT. On the
Windows x86-64 emulator, use x86-64 debug for routine development. ARM64 debug under
the emulator's translation layer crashes in the Dart VM; the ARM64 profile/AOT
integration test passes. See the main guide for its reproduction steps.

Select a feature and tap **Generate PDF**. **Run all tests** executes the 22 native
fixtures and exposes the full report. The PDF preview has page navigation controls.

To run the full UI test on an x86-64 emulator:

```powershell
flutter build apk --debug --target-platform android-x64 --target integration_test/app_test.dart
flutter drive --driver=test_driver/integration_test.dart --target=integration_test/app_test.dart --use-application-binary=build/app/outputs/flutter-apk/app-debug.apk -d emulator-5554
```

The test checks the actual .NET runtime/architecture, all feature results, every
rendered page, links, error recovery and preview navigation. Screenshots and a report
are saved in `../../artifacts/emulator/`. These are private test builds signed with
the Android debug key.

Changing the target platform also filters packaged JNI libraries, so single-ABI test
APKs do not advertise an ABI for which their Flutter engine is absent.
