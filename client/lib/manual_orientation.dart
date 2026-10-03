import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// Only user button presses change orientation. Sensor/window resize does not.
class ManualOrientation extends ChangeNotifier {
  bool landscape = false;
  final Future<void> Function(List<DeviceOrientation>) apply;
  ManualOrientation({Future<void> Function(List<DeviceOrientation>)? apply})
    : apply =
          apply ??
          ((orientations) => Platform.isAndroid || Platform.isIOS
              ? SystemChrome.setPreferredOrientations(orientations)
              : Future<void>.value());
  Future<void> initialize() => apply([DeviceOrientation.portraitUp]);
  Future<void> toggle() async {
    final next = !landscape;
    await apply([
      next ? DeviceOrientation.landscapeLeft : DeviceOrientation.portraitUp,
    ]);
    landscape = next;
    notifyListeners();
  }
}
