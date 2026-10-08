package com.glucose.app;

import android.content.Intent;
import android.content.res.AssetFileDescriptor;
import android.net.Uri;
import android.os.Bundle;
import android.view.HapticFeedbackConstants;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.PickVisualMediaRequest;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.core.content.IntentCompat;
import com.google.androidgamesdk.GameActivity;
import java.util.ArrayList;
import java.util.List;

/**
 * L'activité de Glucose : elle charge la bibliothèque Rust, dont `android_main` prend ensuite
 * tout en charge — la surface, les doigts, le clavier (fiche 54).
 *
 * Elle ne fait que ce que seul Java peut faire (PARTAGE-1, fiche 56) : ouvrir les fichiers
 * qu'une autre application partage vers Glucose, ou que le sélecteur de photos rend, et les
 * confier à Rust par leur descripteur. Aucune décision ici — ni ce qu'est une image, ni où
 * elle se pose.
 */
public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("glucose_android");
    }

    /**
     * Un partage, confié à Rust : un descripteur par fichier (-1 s'il n'a pas pu s'ouvrir),
     * où le fichier commence dans ce descripteur, sa longueur (-1 : jusqu'au bout), et le texte.
     * Rust prend les descripteurs : c'est lui qui les ferme.
     */
    private static native void recevoirUnPartage(
            int[] fichiers, long[] debuts, long[] longueurs, String texte);

    /**
     * **Le sélecteur de photos du système** (fiche 56) : celui d'Android 13 et après, ou celui
     * que les services de Google y ajoutent depuis Android 4.4 ; à défaut, le sélecteur de
     * documents. Aucune permission de lire tout le stockage. Ce qu'on y choisit suit le chemin
     * d'un partage.
     */
    private final ActivityResultLauncher<PickVisualMediaRequest> selecteur =
            registerForActivityResult(
                    new ActivityResultContracts.PickMultipleVisualMedia(),
                    choisies -> {
                        if (!choisies.isEmpty()) {
                            new Thread(() -> ouvrir(choisies, null), "selecteur").start();
                        }
                    });

    /** Ouvre le sélecteur de photos. Appelé par Rust, du fil de Glucose : on passe à celui de
     *  l'interface, le seul d'où une activité se lance. */
    public void choisirDesImages() {
        runOnUiThread(
                () -> selecteur.launch(
                        new PickVisualMediaRequest.Builder()
                                .setMediaType(
                                        ActivityResultContracts.PickVisualMedia.ImageOnly.INSTANCE)
                                .build()));
    }

    /** **L'appui long a pris** (fiche 57) : le doigt le sent, comme partout sous Android — et
     *  selon le réglage de vibration de l'utilisateur. Appelé par Rust, du fil de Glucose. */
    public void sentirLAppui() {
        runOnUiThread(
                () -> getWindow().getDecorView()
                        .performHapticFeedback(HapticFeedbackConstants.LONG_PRESS));
    }

    @Override
    protected void onCreate(Bundle etat) {
        super.onCreate(etat);
        // Une activité recréée (le système avait tué Glucose), ou rouverte depuis les
        // applications récentes, reçoit l'intention d'origine une seconde fois : le partage
        // ne doit pas se poser deux fois.
        boolean rejouee = (getIntent().getFlags() & Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY) != 0;
        if (etat == null && !rejouee) {
            recevoir(getIntent());
        }
    }

    /** Glucose était déjà ouvert (`singleTask`) : le partage arrive ici. */
    @Override
    protected void onNewIntent(Intent intention) {
        super.onNewIntent(intention);
        setIntent(intention);
        recevoir(intention);
    }

    private void recevoir(Intent intention) {
        String action = intention.getAction();
        List<Uri> adresses = new ArrayList<>();
        if (Intent.ACTION_SEND.equals(action)) {
            Uri une = IntentCompat.getParcelableExtra(intention, Intent.EXTRA_STREAM, Uri.class);
            if (une != null) {
                adresses.add(une);
            }
        } else if (Intent.ACTION_SEND_MULTIPLE.equals(action)) {
            List<Uri> plusieurs = IntentCompat.getParcelableArrayListExtra(
                    intention, Intent.EXTRA_STREAM, Uri.class);
            if (plusieurs != null) {
                adresses.addAll(plusieurs);
            }
        } else {
            return;
        }
        CharSequence texte = intention.getCharSequenceExtra(Intent.EXTRA_TEXT);
        String leTexte = texte == null ? null : texte.toString();
        // Ouvrir un fichier peut attendre (une photo encore dans le nuage) : jamais sur le fil
        // de l'interface. La permission de lire vaut tant que l'activité vit.
        new Thread(() -> ouvrir(adresses, leTexte), "partage").start();
    }

    /** Ouvre chaque fichier comme `openInputStream` l'ouvrirait, et confie le tout à Rust. */
    private void ouvrir(List<Uri> adresses, String texte) {
        int n = adresses.size();
        int[] fichiers = new int[n];
        long[] debuts = new long[n];
        long[] longueurs = new long[n];
        for (int i = 0; i < n; i++) {
            fichiers[i] = -1;
            try (AssetFileDescriptor afd =
                    getContentResolver().openAssetFileDescriptor(adresses.get(i), "r")) {
                if (afd != null) {
                    debuts[i] = afd.getStartOffset();
                    longueurs[i] = afd.getDeclaredLength();
                    fichiers[i] = afd.getParcelFileDescriptor().detachFd();
                }
            } catch (Exception e) {
                // Rust le comptera parmi les illisibles, et le compte-rendu le dira.
            }
        }
        recevoirUnPartage(fichiers, debuts, longueurs, texte);
    }
}
