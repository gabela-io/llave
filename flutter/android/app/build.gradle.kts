plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "es.llave.app"
    compileSdk = 36
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = "es.llave.app"
        minSdk = 24
        targetSdk = 34
        versionCode = flutter.versionCode
        versionName = flutter.versionName

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
}

flutter {
    source = "../.."
}

// ---------------------------------------------------------------------------
// Rust FFI cross-compilation via cargo-ndk
// ---------------------------------------------------------------------------

tasks.register<Exec>("buildRustLibrary") {
    workingDir = file("${rootProject.projectDir}/../..")
    val jniLibsDir = file("${projectDir}/src/main/jniLibs")
    commandLine(
        "sh", "-c",
        "cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 " +
            "-o ${jniLibsDir.absolutePath} build -p llave-core-ffi --release"
    )
    inputs.dir(file("${rootProject.projectDir}/../../crates"))
    inputs.file(file("${rootProject.projectDir}/../../Cargo.toml"))
    inputs.file(file("${rootProject.projectDir}/../../Cargo.lock"))
    outputs.dir(jniLibsDir)
}

tasks.whenTaskAdded {
    if (name == "mergeDebugNativeLibs" || name == "mergeReleaseNativeLibs" ||
        name == "mergeDebugJniLibFolders" || name == "mergeReleaseJniLibFolders") {
        dependsOn("buildRustLibrary")
    }
}
