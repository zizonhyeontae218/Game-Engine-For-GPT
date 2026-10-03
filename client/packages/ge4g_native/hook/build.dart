import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

Future<void> main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) return;
    final code = input.config.code;
    final arch = code.targetArchitecture;
    final os = code.targetOS;
    final triple = switch ((os, arch)) {
      (OS.linux, Architecture.x64) => 'x86_64-unknown-linux-gnu',
      (OS.linux, Architecture.arm64) => 'aarch64-unknown-linux-gnu',
      (OS.windows, Architecture.x64) => 'x86_64-pc-windows-msvc',
      (OS.windows, Architecture.arm64) => 'aarch64-pc-windows-msvc',
      (OS.android, Architecture.arm64) => 'aarch64-linux-android',
      (OS.android, Architecture.arm) => 'armv7-linux-androideabi',
      (OS.android, Architecture.x64) => 'x86_64-linux-android',
      (OS.iOS, Architecture.arm64) =>
        code.iOS.targetSdk == IOSSdk.iPhoneSimulator
            ? 'aarch64-apple-ios-sim'
            : 'aarch64-apple-ios',
      (OS.iOS, Architecture.x64) => 'x86_64-apple-ios',
      _ => throw UnsupportedError('GE4G does not support $os / $arch'),
    };
    final root = input.packageRoot.resolve('../../../');
    if (!File.fromUri(root.resolve('Cargo.toml')).existsSync()) {
      throw StateError('Build the client from its GE4G repository checkout.');
    }
    final environment = <String, String>{};
    if (os == OS.android) {
      final compiler = code.cCompiler?.compiler;
      if (compiler == null) {
        throw StateError(
          'Android NDK compiler was not supplied by Flutter. Install the configured NDK.',
        );
      }
      final ndkTriple = arch == Architecture.arm
          ? 'armv7a-linux-androideabi'
          : triple;
      final suffix = Platform.isWindows ? '.cmd' : '';
      final wrapper = compiler.resolve(
        '$ndkTriple${code.android.targetNdkApi}-clang$suffix',
      );
      if (!File.fromUri(wrapper).existsSync()) {
        throw StateError('Missing Android NDK linker: $wrapper');
      }
      environment['CARGO_TARGET_${triple.replaceAll('-', '_').toUpperCase()}_LINKER'] =
          wrapper.toFilePath();
      // Vendored Lua is C: cc-rs needs the target compiler as well as Rust's linker.
      environment['CC_${triple.replaceAll('-', '_')}'] = wrapper.toFilePath();
      environment['AR_${triple.replaceAll('-', '_')}'] = compiler
          .resolve('llvm-ar')
          .toFilePath();
    }
    if (os == OS.iOS) {
      final sdk = await Process.run('xcrun', [
        '--sdk',
        code.iOS.targetSdk.type,
        '--show-sdk-path',
      ]);
      if (sdk.exitCode != 0) {
        throw StateError('Xcode SDK lookup failed: ${sdk.stderr}');
      }
      environment['SDKROOT'] = sdk.stdout.toString().trim();
      environment['IPHONEOS_DEPLOYMENT_TARGET'] = '${code.iOS.targetVersion}.0';
    }
    final targetDir = input.outputDirectoryShared.resolve('cargo/');
    final result = await Process.run('cargo', [
      'build',
      '--locked',
      '--release',
      '-p',
      'ge4g-client',
      '--target',
      triple,
      '--target-dir',
      targetDir.toFilePath(),
      '--manifest-path',
      root.resolve('Cargo.toml').toFilePath(),
    ], environment: environment);
    if (result.exitCode != 0) {
      throw StateError(
        'Native Basement build failed for $triple. Install the Rust target with rustup target add $triple.\n${result.stdout}\n${result.stderr}',
      );
    }
    final filename = os == OS.windows
        ? 'ge4g_client.dll'
        : os == OS.iOS
        ? 'libge4g_client.dylib'
        : 'libge4g_client.so';
    final library = targetDir.resolve('$triple/release/$filename');
    if (!File.fromUri(library).existsSync()) {
      throw StateError('Cargo did not produce $library');
    }
    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'ge4g_native.dart',
        linkMode: DynamicLoadingBundled(),
        file: library,
      ),
    );
    output.dependencies.addAll([
      root.resolve('Cargo.toml'),
      root.resolve('Cargo.lock'),
    ]);
    for (final file in Directory.fromUri(
      root.resolve('crates/'),
    ).listSync(recursive: true).whereType<File>()) {
      if (file.path.endsWith('.rs') || file.path.endsWith('Cargo.toml')) {
        output.dependencies.add(file.uri);
      }
    }
  });
}
