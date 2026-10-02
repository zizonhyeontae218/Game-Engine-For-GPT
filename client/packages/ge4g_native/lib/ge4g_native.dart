import 'dart:convert';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

const _asset = 'package:ge4g_native/ge4g_native.dart';
@Native<Uint32 Function()>(assetId: _asset, symbol: 'ge4g_abi_version')
external int _abiVersion();
@Native<Pointer<Utf8> Function(Pointer<Utf8>)>(
  assetId: _asset,
  symbol: 'ge4g_request_json',
)
external Pointer<Utf8> _request(Pointer<Utf8> request);
@Native<Void Function(Pointer<Utf8>)>(
  assetId: _asset,
  symbol: 'ge4g_free_string',
)
external void _freeString(Pointer<Utf8> string);
@Native<Int64 Function(Uint64, Pointer<Uint8>, Size)>(
  assetId: _asset,
  symbol: 'ge4g_frame_copy',
)
external int _frameCopy(int session, Pointer<Uint8> destination, int capacity);

/// No borrowed Rust pointers cross the bridge. Every response is owned and freed.
class BasementNative {
  BasementNative() {
    final version = _abiVersion();
    if (version != 1) {
      throw StateError(
        'Basement client ABI $version is unsupported; expected 1.',
      );
    }
  }
  Map<String, dynamic> request(Map<String, dynamic> command) {
    final text = jsonEncode(command).toNativeUtf8();
    Pointer<Utf8> response = nullptr;
    try {
      response = _request(text);
      if (response == nullptr) {
        throw StateError('The native runtime returned no response.');
      }
      final result =
          jsonDecode(response.toDartString()) as Map<String, dynamic>;
      if (result['abi_version'] != 1) {
        throw StateError('Invalid native ABI response.');
      }
      if (result['ok'] != true) {
        throw StateError(
          (result['error'] ?? result['errors'] ?? 'Native operation failed')
              .toString(),
        );
      }
      return result;
    } finally {
      calloc.free(text);
      if (response != nullptr) _freeString(response);
    }
  }

  Uint8List frame(int session, int length) {
    if (length <= 0 || length > 2048 * 2048 * 4) {
      throw StateError('Invalid framebuffer length $length');
    }
    final buffer = calloc<Uint8>(length);
    try {
      final copied = _frameCopy(session, buffer, length);
      if (copied != length) {
        throw StateError('Framebuffer copy failed: $copied of $length bytes');
      }
      return Uint8List.fromList(buffer.asTypedList(length));
    } finally {
      calloc.free(buffer);
    }
  }
}
