// L'application Android de Glucose : une activité Java qui charge `libglucose_android.so`,
// construite par `outils/android/construire.sh` (cargo-ndk) dans `src/main/jniLibs/`.
plugins {
    id("com.android.application")
}

android {
    namespace = "com.glucose.app"
    compileSdk = 36
    // Le NDK avec lequel la bibliothèque Rust est construite : Gradle s'en sert pour lui
    // retirer ses symboles de débogage dans l'APK.
    ndkVersion = "29.0.14206865"

    defaultConfig {
        // La même identité que Glucose sous Windows et que Glucose Tauri.
        applicationId = "com.glucose.app"
        // Android 5.0 (2014) : le plancher de Rust lui-même pour Android. Aucune machine exclue
        // au-dessus : `wgpu` prend Vulkan où il existe (Android 7 et après), OpenGL ES ailleurs.
        minSdk = 21
        targetSdk = 36
        versionCode = 1
        versionName = "2.0.2-dev"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    // Exactement celle qu'attend `android-activity` 0.6.1, la porte de `winit` (fiche 50 § 3).
    implementation("androidx.games:games-activity:4.4.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.core:core:1.16.0")
}
