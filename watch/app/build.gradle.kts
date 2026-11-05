import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    compileSdk = 37
    // Stesso nome di pacchetto dell'app del telefono: il Data Layer di Google collega le due app solo se
    // nome e firma coincidono (in debug la firma e' la chiave di debug di questo Mac, uguale per entrambe).
    namespace = "com.spinlab.dj"
    defaultConfig {
        applicationId = "com.spinlab.dj"
        minSdk = 30
        targetSdk = 36
        versionCode = 1
        versionName = "0.1"
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

dependencies {
    implementation("com.google.android.gms:play-services-wearable:20.0.1")
    implementation("androidx.core:core-ktx:1.19.1")
    implementation("androidx.wear:wear-ongoing:1.1.0")
}
