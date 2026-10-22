# Android ARM64 and x86-64 port

Scope expanded at the user's request to include Android x86-64, so the Windows
x64 emulator can execute and debug the native stack without ARM translation.
Both Android ABIs are built from the same pinned source revisions.

## Native dependency overview

This checkout is QuestPDF 2026.9.1. Layout, pagination and the fluent API are managed C#.
It uses its own C ABI wrappers, not SkiaSharp. Native source is maintained separately;
`src/dotnet/library/MAINTAINING.md` describes the upstream release process.

| Managed caller | Native library | Responsibility | Required ABI version |
| --- | --- | --- | --- |
| `QuestPDF/Skia/*`, `Skia/Text/*` | `libquestpdf_skia.so` | PDF creation, raster/SVG output, images, drawing, paragraphs, font registration and subsetting, tagged PDF | 26 |
| `QuestPDF/Qpdf/QpdfAPI.cs` | `libquestpdf_qpdf.so` | Merge/select/rotate/overlay pages, encryption, attachments, XMP document operations | 4 |
| `Helpers/NativeRuntimeDetection.cs` | desktop `libc` probe | Distinguish glibc from musl; Android must be recognized before this probe | n/a |

Skia statically incorporates its codecs and shaping dependencies: FreeType, HarfBuzz,
libgrapheme, an ICU bidi subset, libpng, libjpeg-turbo, libwebp, Wuffs, zlib, Expat and the Skia PDF/SVG/paragraph
modules. The Android wrapper links Android system libraries and a static C++ runtime.
Qpdf uses Android zlib and its built-in crypto provider (no application OpenSSL needed
for qpdf). These are actual builds of the QuestPDF forks: renaming Linux/glibc ARM64
binaries or substituting stock Skia/qpdf would not provide the required ABI.

## Necessary changes and additions

1. Build pinned QuestPDF Skia and qpdf C++ sources with Android NDK for `arm64-v8a` and `x86_64`,
   with PIC, hidden internal symbols, Android-compatible font discovery, and 16 KB
   ELF load-segment alignment. Preserve all managed-imported exports and ABI versions.
2. Detect Android and the NativeAOT Bionic RID separately from Linux/musl. Package
   native assets for `android-arm64` / `android-x64` (.NET Android) and `linux-bionic-arm64` / `linux-bionic-x64`
   (standalone NativeAOT) and avoid selecting desktop Linux assets.
3. Replace custom UTF-8 P/Invoke marshaling with built-in UTF-8 marshaling for modern
   .NET while keeping the netstandard compatibility path.
4. Supply the default Lato archive as an embedded library resource (loose-file loading
   remains available), embed the test's Arabic font, and configure a writable app
   cache directory for qpdf temporary files. APK assets are not ordinary loose files.
5. Export a small versioned C API from a .NET NativeAOT shared library. Return owned
   buffers with explicit free, stable status codes, and caught exception messages.
6. Package the bridge and both dependencies into Flutter `jniLibs/{arm64-v8a,x86_64}`, call via
   Dart FFI away from the UI isolate, and preview returned bytes with `pdfrx`.
7. Validate ABI/exports/dependencies/alignment, managed regressions, Android feature
   generation, independent PDF parsing/rendering, and the actual Flutter preview flow.

## Implementation plan

- [x] Build tools: pin source commits, synchronize Skia dependencies, build both native
  libraries reproducibly and inspect generated ELF files.
- [x] Managed support: Android runtime detection, AOT-safe interop and native packaging.
- [x] NativeAOT test bridge: feature fixtures, manifest/results, PDF byte ownership and
  error/repeated-generation tests.
- [x] Flutter app: generation controls, status/results and pdfrx viewer, integration test.
- [x] Emulator: execute all native fixtures for both ABIs; render and navigate the x86-64 and ARM64 Flutter previews.
- [x] Verification report and final dual-ABI demo APK.

## Validation matrix

Essential coverage includes basic PDF/metadata; multipage rows/columns/tables and
repeated headers/footers; page numbering; rich Latin text and embedded fonts; Arabic
RTL and mixed text; PNG/JPEG/WebP; SVG (paths and text); backgrounds/borders/rounded
corners/shadows/transforms/layers; links/bookmarks; dynamic/lazy content; image and SVG
exports; qpdf merge/select/overlay/underlay; attachments; encryption/decryption; tagged
PDF and PDF/A generation; system-font discovery; expected layout errors; repeated and concurrent generation.
Formal PDF/A and PDF/UA compliance requires a validator such as veraPDF; successful
generation or viewing alone is not compliance certification.

## Platform constraints discovered locally

The Windows x64 emulator refuses a pure ARM64 system image. The installed API 36.1
x86_64 Google Play image advertises `arm64-v8a` and `libndk_translation.so`. Validation
uses native x86-64 execution for the main Flutter test and ARM64 translation for the
ARM64 C ABI test runner. This exercises ARM64 ELF code on Android, but does not replace physical
ARM64 hardware testing or a native ARM64-host emulator.

Standalone NativeAOT uses `linux-bionic-arm64` / `linux-bionic-x64`; `android-arm64` / `android-x64` are the .NET Android
workload RID. Android NativeAOT is experimental. The bridge targets .NET 10 and avoids
JNI because Dart FFI calls its exported functions directly.

## References

- [QuestPDF Skia wrapper/build sources](https://github.com/QuestPDF/QuestPDF.Native.Skia.Build)
- [QuestPDF Skia fork](https://github.com/QuestPDF/QuestPDF.Native.Skia)
- [QuestPDF qpdf build sources](https://github.com/QuestPDF/QuestPDF.Native.Qpdf.Build)
- [.NET NativeAOT on Android Bionic](https://github.com/dotnet/runtime/blob/release/10.0/src/coreclr/nativeaot/docs/android-bionic.md)
- [Android 16 KB page sizes](https://developer.android.com/guide/practices/page-sizes)

## Build and run

On Windows, prerequisites are .NET 10 SDK, Flutter, Android SDK with NDK r29 and
CMake/Ninja, Python 3, Git and a host C compiler (`gcc` on PATH). The host compiler
generates libgrapheme's Unicode 15 tables before cross-compilation. Windows Skia GN
otherwise skips these generated headers. Source commits are pinned in
`native/android/sources.json`; upstream checkouts and build products live in ignored
`artifacts/`. Start with two compiler jobs to bound memory use.

```powershell
python scripts/build_android_native.py --abi arm64-v8a --ndk C:/Android/ndk/29.0.14206865 --cmake C:/Android/cmake/3.22.1/bin/cmake.exe --ninja C:/Android/cmake/3.22.1/bin/ninja.exe
python scripts/build_android_native.py --abi x86_64 --ndk C:/Android/ndk/29.0.14206865 --cmake C:/Android/cmake/3.22.1/bin/cmake.exe --ninja C:/Android/cmake/3.22.1/bin/ninja.exe
./scripts/publish_android_bridge.ps1 -Abi arm64-v8a
./scripts/publish_android_bridge.ps1 -Abi x86_64

cd samples/flutter_android
flutter pub get
flutter build apk --debug --target-platform android-arm64,android-x64
```

For a distributable local demo that uses Dart AOT too, use `--profile` in place of
`--debug`. The supplied demo APK is a profile build signed with the debug key.

For the emulator integration test, build the test entry point for the selected ABI,
then run the existing APK. The integration runner asserts the executed .NET ABI,
requires every native test to pass, opens and renders every generated page with
PDFium/pdfrx, checks links, tests recovery after a managed exception, and navigates the
seven-page table preview. The driver saves results and a screenshot under
`artifacts/emulator/`.

For ARM64 on this x86-64 emulator, the Flutter debug/JIT build crashes in
`libflutter.so` under `libndk_translation` before PDF generation. The standalone
ARM64 .NET runner succeeds. An ARM64 Flutter **profile/AOT** build also passes the
entire integration test. To reproduce it, build the test with `--profile
--target-platform android-arm64`, install with `adb install -t -r --abi arm64-v8a`,
start the activity with `adb shell am start`, forward the Dart VM service port from
logcat, and run the host driver with `VM_SERVICE_URL` set to that forwarded service
URL. The profile test runs automatically on launch; attaching the driver retrieves
the saved report and screenshot. This is an emulator translation boundary; a
physical ARM64 phone has not been tested.

```powershell
flutter build apk --debug --target-platform android-x64 --target integration_test/app_test.dart
flutter drive --driver=test_driver/integration_test.dart --target=integration_test/app_test.dart --use-application-binary=build/app/outputs/flutter-apk/app-debug.apk -d emulator-5554
```

`scripts/verify_android_binaries.py` checks ELF machine, all LOAD segment alignments,
absence of glibc dependencies, and every imported managed export. It also checks the
four bridge exports. `scripts/verify_feature_pdfs.py` independently parses all fixture
PDFs with pypdf and renders every page with Poppler, checking embedded fonts, metadata,
links, page selection/merging, attachments, AES passwords and raster/SVG exports.

## Results measured on 2026-10-22

| Check | Result |
| --- | --- |
| Existing managed unit tests | 457 passed |
| Existing managed layout tests | 102 passed |
| Framework builds | net10.0, net8.0, net6.0, net5.0, netstandard2.0 built in local NuGet pack |
| ARM64 native C ABI, API 36.1 translation | 22/22 feature tests passed |
| x86-64 native C ABI, API 36.1 native execution | 22/22 feature tests passed |
| Independent verification per Android ABI | 22 PDFs / 36 pages parsed and rendered successfully |
| x86-64 Flutter integration | Passed generation, all features, all-page rendering, links, exception recovery, page navigation and screenshot |
| ARM64 Flutter integration, profile/AOT under translation | Passed the same complete UI test; 22 feature checks and 36 rendered pages |
| Flutter analysis / widget smoke test | No analyzer issues; widget test passed |
| Native ELF checks per ABI | Skia: 100 exports, qpdf: 14 exports, bridge: 4 exports; all LOAD segments aligned to 16 KB |
| NuGet artifact | `artifacts/packages/QuestPDF.2026.9.1-android.1.nupkg`; five managed TFMs and four Android/Bionic RIDs |
| Final dual-ABI profile APK | 18 native libraries, all ELF LOAD alignments >= 16 KB; `zipalign -c -P 16 -v 4` passed; APK v2 signature verified |

Native assets inside the NuGet archive were compared byte-for-byte with the verified
ELF binaries for all four RIDs. Per-ABI integration reports and screenshots are in
`artifacts/emulator/`. Independent native-runner PDFs and rendered pages are in
`artifacts/emulator-arm64/` and `artifacts/emulator-x64/`.

Final deliverables are in `artifacts/deliverables/`: the dual-ABI profile APK, the
local NuGet package, a preview screenshot and `verification.json` with checksums,
feature counts and evidence boundaries. The final normal app was also installed
and launched on the emulator using its x86-64 ABI.

The local NuGet build disables the upstream baseline package validation; it does not
claim a complete API compatibility audit. Native binaries are built against Android
API 24; runtime verification here is API 36.1. PDF/A and PDF/UA fixtures are generated
and their structural metadata is inspected; formal veraPDF certification has not run.
Physical ARM64 hardware and a real 16 KB-page Android kernel are not tested here.

The machine's D: drive filled during compilation. Generated dependency/build trees,
.NET output folders and Flutter build output were preserved and moved to
`F:/CodexBuilds/QuestPdfAndroid`, with directory junctions at their original workspace
locations. Source files remain in this repository. No global caches or user files
were deleted. These junctions are local build storage and are not Git artifacts.
