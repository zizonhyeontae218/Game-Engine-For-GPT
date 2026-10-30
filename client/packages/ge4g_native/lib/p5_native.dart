import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

const _asset = 'package:ge4g_native/p5_native.dart';

@Native<Uint32 Function()>(assetId: _asset, symbol: 'ge4g_p5_v1_abi_version')
external int _abiVersion();
@Native<Pointer<Utf8> Function(Pointer<Uint8>, Size)>(
  assetId: _asset,
  symbol: 'ge4g_p5_v1_request',
)
external Pointer<Utf8> _request(Pointer<Uint8> request, int length);
@Native<Void Function(Pointer<Utf8>)>(
  assetId: _asset,
  symbol: 'ge4g_p5_v1_string_free',
)
external void _freeString(Pointer<Utf8> response);

abstract interface class PentominoBridge {
  /// Calls must stay on the session's creating isolate/thread.
  dynamic request(Map<String, dynamic> command);
}

class PentominoFailure implements Exception {
  final String code;
  final String detail;
  const PentominoFailure(this.code, this.detail);
  @override
  String toString() => '$code: $detail';
}

/// An independent additive native asset; FlatLand's ABI remains unchanged.
class PentominoNative implements PentominoBridge {
  PentominoNative() {
    if (_abiVersion() != 1) {
      throw StateError('Unsupported Pentomino native ABI.');
    }
  }

  @override
  dynamic request(Map<String, dynamic> command) {
    final bytes = utf8.encode(jsonEncode({'version': 1, 'command': command}));
    final input = calloc<Uint8>(bytes.length);
    Pointer<Utf8> response = nullptr;
    try {
      input.asTypedList(bytes.length).setAll(0, bytes);
      response = _request(input, bytes.length);
      if (response == nullptr) throw StateError('No native response.');
      final body = jsonDecode(response.toDartString()) as Map<String, dynamic>;
      if (body['version'] != 1) throw StateError('Invalid native response.');
      if (body['ok'] != true) {
        final error = body['error'] as Map<String, dynamic>;
        throw PentominoFailure(
          error['code'] as String,
          error['detail'] as String,
        );
      }
      return body['result'];
    } finally {
      calloc.free(input);
      if (response != nullptr) _freeString(response);
    }
  }
}
