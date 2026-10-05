plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val ge4gKeystore = System.getenv("GE4G_ANDROID_KEYSTORE")

android {
    namespace = "dev.ge4g.ge4g_client"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "dev.ge4g.ge4g_client"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (ge4gKeystore != null) {
            create("ge4gRelease") {
                storeFile = file(ge4gKeystore)
                storePassword = requireNotNull(System.getenv("GE4G_ANDROID_STORE_PASSWORD"))
                keyAlias = "ge4g"
                keyPassword = storePassword
                val keyStore = java.security.KeyStore.getInstance("PKCS12")
                storeFile!!.inputStream().use { keyStore.load(it, storePassword!!.toCharArray()) }
                val certificate = requireNotNull(keyStore.getCertificate(keyAlias))
                val fingerprint = java.security.MessageDigest.getInstance("SHA-256")
                    .digest(certificate.encoded).joinToString("") { "%02x".format(it.toInt() and 0xff) }
                require(fingerprint == rootProject.file("signing-certificate.sha256").readText().trim()) {
                    "GE4G release certificate changed; refusing an incompatible Android update"
                }
            }
        }
    }

    buildTypes {
        release {
            // Hosted builds are unsigned; delivery signs with the preserved private key.
            // Never create a runner-specific debug certificate for a release again.
            signingConfig = if (ge4gKeystore != null) signingConfigs.getByName("ge4gRelease") else null
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
