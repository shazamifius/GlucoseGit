// L'application Android de Glucose : une activité Java qui charge `libglucose_android.so`,
// construite par `outils/android/construire.sh` (cargo-ndk) dans `src/main/jniLibs/`.
plugins {
    id("com.android.application")
}

// **La version** (fiche 54) : celle que la publication donne, ou celle du travail.
val versionDeGlucose: String = System.getenv("GLUCOSE_VERSION") ?: "2.0.2-dev"

/**
 * **Un code qui monte avec la version** : Android refuse une mise à jour dont le code ne monte
 * pas. `x.y.z` vaut `((x·100 + y)·100 + z)·1000`, plus le rang de la préversion — `alpha.n`
 * 100 + n, `beta.n` 200 + n, `rc.n` 300 + n, la version finale 999, une version de travail 0 :
 * `2.0.2-beta.1` < `2.0.2-beta.2` < `2.0.2` < `2.0.3-dev`.
 */
fun codeDeVersion(v: String): Int {
    val morceaux = v.split("-", limit = 2)
    val (x, y, z) = morceaux[0].split(".").map { it.toInt() }
    val pre = morceaux.getOrNull(1)
    val n = pre?.substringAfter(".", "0")?.toIntOrNull() ?: 0
    val rang = when {
        pre == null -> 999
        pre.startsWith("rc") -> 300 + n
        pre.startsWith("beta") -> 200 + n
        pre.startsWith("alpha") -> 100 + n
        else -> 0
    }
    return ((x * 100 + y) * 100 + z) * 1000 + rang
}

// **La clé de publication** (fiche 54) : donnée par la publication sur GitHub (ses secrets), ou
// par qui construit avec elle. Sans elle, seul l'APK de travail se construit, signé de la clé de
// débogage de la machine.
val cleDePublication: String? = System.getenv("GLUCOSE_KEYSTORE")

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
        versionCode = codeDeVersion(versionDeGlucose)
        versionName = versionDeGlucose
    }

    signingConfigs {
        if (cleDePublication != null) {
            create("publication") {
                storeFile = file(cleDePublication)
                storePassword = System.getenv("GLUCOSE_KEYSTORE_PASSWORD")
                keyAlias = "glucose"
                keyPassword = System.getenv("GLUCOSE_KEYSTORE_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (cleDePublication != null) {
                signingConfig = signingConfigs.getByName("publication")
            }
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
