package com.glucose.app;

import android.app.PendingIntent;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.pm.PackageInstaller;
import android.content.res.AssetFileDescriptor;
import android.graphics.Color;
import android.net.Uri;
import android.os.BatteryManager;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.view.HapticFeedbackConstants;
import android.view.View;
import androidx.activity.EdgeToEdge;
import androidx.activity.SystemBarStyle;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.PickVisualMediaRequest;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.core.content.ContextCompat;
import androidx.core.content.IntentCompat;
import androidx.core.graphics.Insets;
import androidx.core.view.WindowInsetsCompat;
import com.google.androidgamesdk.GameActivity;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;
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
     * Où en est une mise à jour confiée à Android (MAJ-ANDROID-1, fiche 58) : 1, il faut
     * autoriser Glucose à installer des applications ; 2, Android demande la confirmation ;
     * 3, l'installation a échoué, et `detail` dit pourquoi. Rust choisit les mots.
     */
    private static native void recevoirLInstallation(int etat, String detail);

    /** L'action de l'intention par laquelle Android dit où en est l'installation. */
    private static final String INSTALLATION = "com.glucose.app.INSTALLATION";

    /** L'APK qui attend que l'utilisateur autorise Glucose à installer : il repart au retour. */
    private String apkEnAttente;

    /** Les marges du système, en pixels : les barres et l'encoche, puis le clavier (BORD-1). */
    private static native void recevoirLesMarges(
            int gauche, int haut, int droite, int bas, int clavier);

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

    /**
     * **La batterie**, telle qu'Android la dit à tous (l'intention qu'il garde,
     * `ACTION_BATTERY_CHANGED`) : le niveau, l'échelle, et la prise qui la charge. Rust en fait
     * un pourcentage (fiche 58). `null` si le système ne dit rien. Appelé par Rust, du fil du
     * journal technique.
     */
    public int[] lireLaBatterie() {
        Intent etat = ContextCompat.registerReceiver(
                this, null, new IntentFilter(Intent.ACTION_BATTERY_CHANGED),
                ContextCompat.RECEIVER_NOT_EXPORTED);
        if (etat == null) {
            return null;
        }
        return new int[] {
            etat.getIntExtra(BatteryManager.EXTRA_LEVEL, -1),
            etat.getIntExtra(BatteryManager.EXTRA_SCALE, -1),
            etat.getIntExtra(BatteryManager.EXTRA_PLUGGED, -1),
        };
    }

    /** **L'appui long a pris** (fiche 57) : le doigt le sent, comme partout sous Android — et
     *  selon le réglage de vibration de l'utilisateur. Appelé par Rust, du fil de Glucose. */
    public void sentirLAppui() {
        runOnUiThread(
                () -> getWindow().getDecorView()
                        .performHapticFeedback(HapticFeedbackConstants.LONG_PRESS));
    }

    /**
     * **Confie une mise à jour à Android** (MAJ-ANDROID-1, fiche 58) : l'APK est déjà vérifié
     * par Rust — sa signature de Glucose — ; Android vérifie la sienne, celle de la clé de
     * Glucose pour Android, puis demande à l'utilisateur et remplace Glucose. Appelé par Rust,
     * du fil qui a téléchargé.
     */
    public void installerUnApk(String chemin) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O
                && !getPackageManager().canRequestPackageInstalls()) {
            // Une fois pour toutes : la page où l'utilisateur autorise Glucose. Au retour
            // (`onResume`), l'installation repart.
            apkEnAttente = chemin;
            recevoirLInstallation(1, null);
            runOnUiThread(() -> startActivity(new Intent(
                    Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                    Uri.parse("package:" + getPackageName()))));
            return;
        }
        new Thread(() -> confier(chemin), "installation").start();
    }

    /** Écrit l'APK dans une session de `PackageInstaller`, et la remet au système. */
    private void confier(String chemin) {
        try {
            PackageInstaller installeur = getPackageManager().getPackageInstaller();
            PackageInstaller.SessionParams reglages =
                    new PackageInstaller.SessionParams(
                            PackageInstaller.SessionParams.MODE_FULL_INSTALL);
            reglages.setAppPackageName(getPackageName());
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                // Sans confirmation là où Android le permet (12 et après) : une application
                // qui se met à jour elle-même, et le déclare (le manifeste). L'accord est déjà
                // donné, à la question de Glucose. Sinon, Android demande.
                reglages.setRequireUserAction(
                        PackageInstaller.SessionParams.USER_ACTION_NOT_REQUIRED);
            }
            int numero = installeur.createSession(reglages);
            File apk = new File(chemin);
            try (PackageInstaller.Session session = installeur.openSession(numero)) {
                try (InputStream lu = new FileInputStream(apk);
                        OutputStream ecrit = session.openWrite("glucose.apk", 0, apk.length())) {
                    byte[] tampon = new byte[1 << 16];
                    int n;
                    while ((n = lu.read(tampon)) > 0) {
                        ecrit.write(tampon, 0, n);
                    }
                    session.fsync(ecrit);
                }
                Intent retour = new Intent(this, MainActivity.class).setAction(INSTALLATION);
                int drapeaux = PendingIntent.FLAG_UPDATE_CURRENT;
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    // Le système écrit l'état dans l'intention : elle doit pouvoir changer.
                    drapeaux |= PendingIntent.FLAG_MUTABLE;
                }
                PendingIntent suite = PendingIntent.getActivity(this, 0, retour, drapeaux);
                session.commit(suite.getIntentSender());
            }
        } catch (Exception e) {
            recevoirLInstallation(3, e.toString());
        }
    }

    /** Ce qu'Android dit de l'installation : demander, ou dire pourquoi elle a échoué. */
    private void suivreLInstallation(Intent intention) {
        int etat = intention.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE);
        if (etat == PackageInstaller.STATUS_PENDING_USER_ACTION) {
            Intent demande = IntentCompat.getParcelableExtra(intention, Intent.EXTRA_INTENT, Intent.class);
            if (demande != null) {
                recevoirLInstallation(2, null);
                startActivity(demande);
            }
        } else if (etat != PackageInstaller.STATUS_SUCCESS) {
            recevoirLInstallation(3, intention.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE));
        }
    }

    /** De retour de la page qui autorise Glucose à installer : l'APK en attente repart. */
    @Override
    protected void onResume() {
        super.onResume();
        if (apkEnAttente != null
                && (Build.VERSION.SDK_INT < Build.VERSION_CODES.O
                        || getPackageManager().canRequestPackageInstalls())) {
            String chemin = apkEnAttente;
            apkEnAttente = null;
            new Thread(() -> confier(chemin), "installation").start();
        }
    }

    @Override
    protected void onCreate(Bundle etat) {
        // **Bord à bord, partout** (BORD-1, fiche 57) : Android 15 et 16 l'imposent à la cible
        // 36 ; le même chemin sur chaque téléphone. Glucose est sombre : icônes claires.
        EdgeToEdge.enable(
                this,
                SystemBarStyle.dark(Color.TRANSPARENT),
                SystemBarStyle.dark(Color.TRANSPARENT));
        super.onCreate(etat);
        // Une activité recréée (le système avait tué Glucose), ou rouverte depuis les
        // applications récentes, reçoit l'intention d'origine une seconde fois : le partage
        // ne doit pas se poser deux fois.
        boolean rejouee = (getIntent().getFlags() & Intent.FLAG_ACTIVITY_LAUNCHED_FROM_HISTORY) != 0;
        if (etat == null && !rejouee) {
            recevoir(getIntent());
        }
    }

    /** **Les marges du système** (BORD-1) : `GameActivity` en fait ce qu'elle en fait, et Rust
     *  les reçoit — la barre d'état, la navigation, l'encoche, et le clavier, qui recouvre la
     *  fenêtre au lieu de la rétrécir en bord à bord. */
    @Override
    public WindowInsetsCompat onApplyWindowInsets(View vue, WindowInsetsCompat marges) {
        WindowInsetsCompat rendu = super.onApplyWindowInsets(vue, marges);
        Insets barres = marges.getInsets(
                WindowInsetsCompat.Type.systemBars() | WindowInsetsCompat.Type.displayCutout());
        Insets clavier = marges.getInsets(WindowInsetsCompat.Type.ime());
        recevoirLesMarges(barres.left, barres.top, barres.right, barres.bottom, clavier.bottom);
        return rendu;
    }

    /** Glucose était déjà ouvert (`singleTask`) : le partage arrive ici. */
    @Override
    protected void onNewIntent(Intent intention) {
        super.onNewIntent(intention);
        if (INSTALLATION.equals(intention.getAction())) {
            suivreLInstallation(intention);
            return;
        }
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
