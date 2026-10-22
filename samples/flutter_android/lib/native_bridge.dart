import 'dart:convert';
import 'dart:ffi';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';

const featureNames = [
  'Overview and metadata', 'Multipage table and pagination',
  'Arabic RTL and rich text', 'PNG JPEG WebP images', 'SVG paths and text',
  'Styled boxes layers transforms', 'Links and destinations',
  'Dynamic and lazy content', 'PDF/A-3b generation', 'Tagged PDF/UA generation',
  'Merge and select pages', 'Overlay and underlay', 'Unicode attachment',
  'AES-128 encryption roundtrip', 'AES-256 encryption roundtrip',
  'PNG JPEG WebP raster exports', 'SVG export', 'Expected layout failure',
  'Repeated generation', 'Concurrent generation', 'XMP metadata extension', 'System fonts',
];

typedef _GenerateNative = Int32 Function(Int32, Pointer<Utf8>, Pointer<Pointer<Uint8>>, Pointer<Int32>);
typedef _GenerateDart = int Function(int, Pointer<Utf8>, Pointer<Pointer<Uint8>>, Pointer<Int32>);
typedef _TestsNative = Int32 Function(Pointer<Utf8>, Pointer<Pointer<Uint8>>, Pointer<Int32>);
typedef _TestsDart = int Function(Pointer<Utf8>, Pointer<Pointer<Uint8>>, Pointer<Int32>);

class NativeResult {
  const NativeResult(this.status, this.bytes);
  final int status;
  final Uint8List bytes;
  String get text => utf8.decode(bytes);
}

/// Copy results before freeing them with the allocator in the owning .NET library.
NativeResult invokeNative((int, String) request) {
  DynamicLibrary.open('libquestpdf_skia.so');
  DynamicLibrary.open('libquestpdf_qpdf.so');
  final library = DynamicLibrary.open('libquestpdf_android.so');
  final version = library.lookupFunction<Int32 Function(), int Function()>('questpdf_abi_version');
  if (version() != 1) throw StateError('Unsupported QuestPDF bridge ABI');
  final free = library.lookupFunction<Void Function(Pointer<Void>), void Function(Pointer<Void>)>('questpdf_free');
  final path = request.$2.toNativeUtf8();
  final output = calloc<Pointer<Uint8>>();
  final length = calloc<Int32>();
  try {
    final status = request.$1 == -1
      ? library.lookupFunction<_TestsNative, _TestsDart>('questpdf_run_tests')(path, output, length)
      : library.lookupFunction<_GenerateNative, _GenerateDart>('questpdf_generate')(request.$1, path, output, length);
    if (length.value < 0 || (length.value != 0 && output.value == nullptr)) {
      throw StateError('Invalid buffer returned by QuestPDF');
    }
    return NativeResult(status, Uint8List.fromList(
      length.value == 0 ? <int>[] : output.value.asTypedList(length.value)));
  } finally {
    if (output.value != nullptr) free(output.value.cast());
    calloc.free(length);
    calloc.free(output);
    calloc.free(path);
  }
}

Future<NativeResult> generateDocument(int feature, String directory) => compute(invokeNative, (feature, directory));
Future<NativeResult> runFeatureSuite(String directory) => compute(invokeNative, (-1, directory));
