package com.glucose.app;

import com.google.androidgamesdk.GameActivity;

/**
 * L'activité de Glucose : elle ne fait que charger la bibliothèque Rust, dont `android_main`
 * prend ensuite tout en charge — la surface, les doigts, le clavier (fiche 54).
 */
public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("glucose_android");
    }
}
