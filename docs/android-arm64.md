# Android ARMv7, ARM64 and x86-64 port

Scope expanded at the user's request to include Android x86-64, so the Windows
x64 emulator can execute and debug the native stack without ARM translation.
ARMv7 (`armeabi-v7a`) was added subsequently. All Android ABIs are built from the
same pinned source revisions. Historical results below describe the original two-ABI
verification; the ARMv7 extension is tracked separately.

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

1. Build pinned QuestPDF Skia and qpdf C++ sources with Android NDK for `armeabi-v7a`, `arm64-v8a` and `x86_64`,
   with PIC, hidden internal symbols, Android-compatible font discovery, and 16 KB
   ELF load-segment alignment. Preserve all managed-imported exports and ABI versions.
2. Detect Android and the NativeAOT Bionic RID separately from Linux/musl. Package
   native assets for `android-arm` / `android-arm64` / `android-x64` (.NET Android) and `linux-bionic-arm` / `linux-bionic-arm64` / `linux-bionic-x64`
   (standalone NativeAOT) and avoid selecting desktop Linux assets.
3. Replace custom UTF-8 P/Invoke marshaling with built-in UTF-8 marshaling for modern
   .NET while keeping the netstandard compatibility path.
4. Supply the default Lato archive as an embedded library resource (loose-file loading
   remains available), embed the test's Arabic font, and configure a writable app
   cache directory for qpdf temporary files. APK assets are not ordinary loose files.
5. Export a small versioned C API from a .NET NativeAOT shared library. Return owned
   buffers with explicit free, stable status codes, and caught exception messages.
6. Package the bridge and both dependencies into Flutter `jniLibs/{armeabi-v7a,arm64-v8a,x86_64}`, call via
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
ARM64 hardware testing or a native ARM64-host emulator. Subsequently, the repository
user reported testing ARM64 on a physical HONOR X70 running Android 16; automated
suite results and kernel page size were not supplied with that report.

Standalone NativeAOT uses `linux-bionic-arm64` / `linux-bionic-x64`; `android-arm64` / `android-x64` are the .NET Android
workload RID. Android NativeAOT is experimental. The bridge targets .NET 10 and avoids
JNI because Dart FFI calls its exported functions directly.

## References

- [QuestPDF Skia wrapper/build sources](https://github.com/QuestPDF/QuestPDF.Native.Skia.Build)
- [QuestPDF Skia fork](https://github.com/QuestPDF/QuestPDF.Native.Skia)
- [QuestPDF qpdf build sources](https://github.com/QuestPDF/QuestPDF.Native.Qpdf.Build)
- [.NET NativeAOT on Android Bionic](https://github.com/dotnet/runtime/blob/release/10.0/src/coreclr/nativeaot/docs/android-bionic.md)
- [.NET NativeAOT ARM32 Bionic build support](https://github.com/dotnet/runtime/pull/97996)
- [Android emulator ARM translation](https://android-developers.googleblog.com/2020/03/run-arm-apps-on-android-emulator.html)
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
python scripts/build_android_native.py --abi armeabi-v7a --ndk C:/Android/ndk/29.0.14206865 --cmake C:/Android/cmake/3.22.1/bin/cmake.exe --ninja C:/Android/cmake/3.22.1/bin/ninja.exe
./scripts/publish_android_bridge.ps1 -Abi arm64-v8a
./scripts/publish_android_bridge.ps1 -Abi x86_64
./scripts/publish_android_bridge.ps1 -Abi armeabi-v7a

cd samples/flutter_android
flutter pub get
flutter build apk --profile --target-platform android-arm,android-arm64,android-x64
```

The profile build uses Dart AOT too. For local debugging, use `--debug` in place of
`--profile`. The supplied demo APK is a profile build signed with the debug key.

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
the saved report and screenshot. This is an emulator translation boundary. Physical
ARM64 testing was subsequently reported by the repository user on a HONOR X70
running Android 16, separately from the automated emulator results.

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

This table records the initial ARM64/x86-64 validation. See the ARMv7 extension
section below for the subsequent 32-bit work.

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
Physical ARM64 hardware was not part of this initial automated run. The repository
user subsequently reported testing ARM64 on a HONOR X70 running Android 16. A real
16 KB-page Android kernel remains unverified; the device's page size was not reported.

The machine's D: drive filled during compilation. Generated dependency/build trees,
.NET output folders and Flutter build output were preserved and moved to
`F:/CodexBuilds/QuestPdfAndroid`, with directory junctions at their original workspace
locations. Source files remain in this repository. No global caches or user files
were deleted. These junctions are local build storage and are not Git artifacts.

## ARMv7 extension

| Layer | ARMv7 identifier |
| --- | --- |
| Android APK / NDK | `armeabi-v7a` |
| Flutter target | `android-arm` |
| .NET Android package RID | `android-arm` |
| Standalone NativeAOT RID | `linux-bionic-arm` |
| Skia GN CPU / .NET architecture | `arm` / `Arm` |
| ELF | ELF32, ARM machine 40, EABI5, Android soft-float calling convention |

The native sources and minimum Android API 24 are unchanged. The library packages
both Android and Bionic ARM32 native assets. The ELF verifier reads 32-bit and
64-bit program headers separately and rejects ARM hard-float ABI libraries.
The NativeAOT report includes `PointerSize`; native and Flutter integration tests
assert four bytes on ARMv7 and eight on the existing 64-bit targets.

ARM32 Bionic publishes a NativeAOT runtime pack but no CoreCLR runtime pack. The
bridge publishing script applies `PublishAot=true`, `PublishAotUsingRuntimePack=true`
and `TargetFrameworks=net10.0` globally so referenced projects restore against the
NativeAOT pack as well. This only limits the sample's publish operation; the
QuestPDF NuGet package still targets all five original frameworks.

ARMv7 native validation uses the Android 7/API 24 AOSP `armeabi-v7a` system image
on Google's classic emulator 26.1.4. It runs an ARMv7 Linux kernel and emulated
Cortex-A8 CPU with NEON/VFPv3, without an x86 native-bridge translation layer.
Current QEMU2 emulator releases reject ARM32 system images. The classic emulator
was downloaded from [Google's archive](https://dl.google.com/android/repository/emulator-windows-4266726.zip)
and extracted only under ignored task artifacts. The installed Android SDK was
not replaced. Create an AVD for `system-images;android-24;default;armeabi-v7a`
and start it with that emulator's `-engine classic` option.

The API 36.1 image used above has no 32-bit userspace. The Android 11/API 30 Google
APIs x86-64 image does advertise ARMv7 translation, but its `libndk_translation`
crashes in the full JPEG fixture. The identical libraries pass all 22 features
on native ARM emulation, including JPEG. Do not use that translator as the
ARMv7 verification target.

Run the native C ABI suite on the ARM system image:

```powershell
./scripts/test_android_bridge.ps1 -Serial emulator-5558 -Abi armeabi-v7a
```

The script compiles the C runner, pushes the three matching libraries, runs all
22 fixtures and retrieves reports/PDFs under `artifacts/emulator-arm/`. The same
command works with a physical device's serial. `-NativeDirectory` can select an
alternative directory containing all three libraries. `-Loader` is available for
the previously verified ARM64 translation workflow.

For the Flutter test, build from `samples/flutter_android`:

```powershell
flutter build apk --profile --target-platform android-arm --target integration_test/app_test.dart
adb -s emulator-5558 install -t -r --abi armeabi-v7a build/app/outputs/flutter-apk/app-profile.apk
adb -s emulator-5558 shell am start -n com.questpdf.questpdf_android_demo/.MainActivity --ez enable-impeller false --ez enable-software-rendering true
```

The classic emulator exposes GLES 1 through PixelFlinger, so Flutter uses software
rendering for this test. Generation and PDFium/pdfrx page rendering still execute
inside the ARM32 app. The profile test runs automatically. Read its Dart VM service
port and authentication path from logcat, forward that port with `adb forward`,
set `VM_SERVICE_URL` to the forwarded URL, then retrieve the report and screenshot:

```powershell
dart --disable-dart-dev --packages=.dart_tool/package_config.json test_driver/integration_test.dart
```

Direct VM execution avoids running unrelated Windows PDFium native build hooks
for this host-only driver. The test verifies the new document's page count before
navigating, then asserts that the actual preview controller reaches page 2 and
returns to page 1; this avoids a stale-widget race on slower devices.

Alternatively, after building the profile integration APK, run the helper from
the repository root. It installs the APK, launches it, forwards the VM service and
retrieves the results automatically:

```powershell
./scripts/run_android_flutter_test.ps1 -Serial emulator-5558 -Abi armeabi-v7a -SoftwareRendering
```

Native verification on 2026-10-23: 22/22 features passed with runtime
`linux-bionic-arm`, architecture `Arm`, and pointer size 4. Independent pypdf and
Poppler verification parsed and rendered all 22 PDFs / 36 pages. ARM32 ELF checks
passed all 100 Skia, 14 qpdf and four bridge exports, Android-only dependencies,
EABI5 soft-float calling convention and 16 KB LOAD alignment. Managed regressions
passed 457 unit and 102 layout tests; the x86-64 native regression also passed all
22 features on API 30.

`scripts/verify_android_package.py` confirms five managed frameworks and all six
Android/Bionic RIDs in `QuestPDF.2026.9.1-android.2.nupkg`, with all 12 native assets
matching source byte-for-byte. Physical ARMv7 hardware remains untested.

The complete ARMv7 Flutter profile integration also passed on the native API 24
emulator: all 22 native features, all 36 pages rendered with PDFium/pdfrx, links,
managed-error recovery, verified page 2/1 navigation, and a valid preview screenshot.
The report and screenshot are `artifacts/emulator/linux-bionic-arm-integration-results.json`
and `artifacts/emulator/questpdf-arm-preview.png`.

Final deliverables are `artifacts/deliverables/questpdf-android-armv7-arm64-x64-profile.apk`
and `artifacts/deliverables/QuestPDF.2026.9.1-android.2.nupkg`. The normal demo APK
contains 27 native libraries across all three ABIs. Its QuestPDF binaries match
the verified inputs, `zipalign -c -P 16 -v 4` passes, and its APK v2 signature
verifies. The normal demo was also installed and opened its PDF preview on ARMv7.
The APK is a local profile build using the Android debug signing key.

The three QuestPDF native libraries have 16 KB ELF LOAD alignment on every ABI,
and all packaged ARM64/x86-64 dependencies retain 16 KB alignment. The prebuilt
ARM32 `libpdfium.so` supplied by pdfrx has 4 KB ELF alignment; ARMv7 verification
here uses a 4 KB-page kernel. A real 16 KB-page Android kernel and formal PDF/A or
PDF/UA certification remain outside the measured evidence.

`artifacts/deliverables/verification-armv7.json` records checksums, test counts and
these limits. `armv7-apk-verification.json` records each packaged native library's
size and minimum LOAD alignment. The preview screenshot is `android-armv7-preview.png`.
