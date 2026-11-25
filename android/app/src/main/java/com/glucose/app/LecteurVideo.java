package com.glucose.app;

import android.graphics.Bitmap;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.media.AudioAttributes;
import android.media.Image;
import android.media.ImageReader;
import android.media.MediaMetadataRetriever;
import android.media.MediaPlayer;
import android.os.Build;
import android.os.Handler;
import android.os.Looper;
import android.os.SystemClock;
import android.util.Log;
import java.io.FileDescriptor;
import java.io.FileInputStream;

/**
 * **Une vidéo jouée par Android** (VIDEO-ANDROID-1, fiche 172) : `MediaPlayer` décode, cadence
 * les images et joue le son ; ses images arrivent dans un `ImageReader`, que Rust lit plan par
 * plan. Rien ne se décide ici — ni les couleurs, ni le recadrage, ni le quart de tour : Java dit
 * ce que le système dit, Rust en fait l'image.
 *
 * Appelé par Rust, du fil de la vidéo, toujours le même pour un lecteur. Les nouvelles du
 * lecteur (une image, la fin, une faute) arrivent sur le fil de l'interface et ne font que
 * réveiller ce fil-là.
 */
final class LecteurVideo {
    private static final String JOURNAL = "Glucose";
    // Ce que rend `attendre` : les mêmes nombres que `Attente`, côté Rust.
    private static final int RIEN = 0;
    private static final int IMAGE = 1;
    private static final int AU_BOUT = 2;
    private static final int FAUTE = 3;
    /** Les images qu'on peut tenir à la fois : une lue, une qui arrive, une d'avance. */
    private static final int IMAGES = 3;

    private final MediaPlayer joueur;
    private final ImageReader images;
    private final int quarts;
    private final Object signal = new Object();
    private boolean auBout;
    private boolean faute;
    private Image tenue;

    private LecteurVideo(MediaPlayer joueur, ImageReader images, int quarts) {
        this.joueur = joueur;
        this.images = images;
        this.quarts = quarts;
        Handler interface_ = new Handler(Looper.getMainLooper());
        images.setOnImageAvailableListener(r -> reveiller(), interface_);
        joueur.setOnCompletionListener(j -> {
            synchronized (signal) {
                auBout = true;
            }
            reveiller();
        });
        joueur.setOnErrorListener((j, quoi, detail) -> {
            Log.w(JOURNAL, "video : le lecteur a echoue (" + quoi + ", " + detail + ")");
            synchronized (signal) {
                faute = true;
            }
            reveiller();
            return true;
        });
        joueur.setSurface(images.getSurface());
    }

    private void reveiller() {
        synchronized (signal) {
            signal.notifyAll();
        }
    }

    /**
     * **Ouvre la vidéo** de ce fichier, de `debut` sur `longueur` octets (-1 : jusqu'au bout),
     * prête à jouer. `null` si le système ne sait pas la lire.
     */
    static LecteurVideo ouvrir(String chemin, long debut, long longueur, boolean son) {
        MediaPlayer joueur = new MediaPlayer();
        try (FileInputStream fichier = new FileInputStream(chemin)) {
            FileDescriptor fd = fichier.getFD();
            long l = longueur >= 0 ? longueur : fichier.getChannel().size() - debut;
            joueur.setDataSource(fd, debut, l);
            joueur.setAudioAttributes(new AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                    .build());
            joueur.prepare();
            int largeur = joueur.getVideoWidth();
            int hauteur = joueur.getVideoHeight();
            if (largeur <= 0 || hauteur <= 0) {
                joueur.release();
                return null;
            }
            ImageReader images =
                    ImageReader.newInstance(largeur, hauteur, ImageFormat.YUV_420_888, IMAGES);
            LecteurVideo lecteur = new LecteurVideo(joueur, images, quarts(fd, debut, l));
            lecteur.couperLeSon(!son);
            return lecteur;
        } catch (Exception e) {
            Log.w(JOURNAL, "video : ne s'ouvre pas : " + e);
            joueur.release();
            return null;
        }
    }

    /** Les quarts de tour que la vidéo demande (ses métadonnées) ; 0 si elle ne dit rien. */
    private static int quarts(FileDescriptor fd, long debut, long longueur) {
        MediaMetadataRetriever lu = new MediaMetadataRetriever();
        try {
            lu.setDataSource(fd, debut, longueur);
            String angle =
                    lu.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_ROTATION);
            return angle == null ? 0 : Integer.parseInt(angle) / 90;
        } catch (Exception e) {
            return 0;
        } finally {
            relacher(lu);
        }
    }

    private static void relacher(MediaMetadataRetriever lu) {
        try {
            lu.release();
        } catch (Exception e) {
            // `release` déclare une `IOException` depuis Android 10 ; il n'y a rien à reprendre.
        }
    }

    /**
     * **La première image** de la vidéo de ce fichier, tournée comme elle se regarde : sa
     * largeur, sa hauteur, puis chaque pixel en ARGB. `null` si le système ne sait pas la lire.
     */
    static int[] imageFixe(String chemin, long debut, long longueur) {
        MediaMetadataRetriever lu = new MediaMetadataRetriever();
        try (FileInputStream fichier = new FileInputStream(chemin)) {
            long l = longueur >= 0 ? longueur : fichier.getChannel().size() - debut;
            lu.setDataSource(fichier.getFD(), debut, l);
            Bitmap image = lu.getFrameAtTime(0, MediaMetadataRetriever.OPTION_CLOSEST_SYNC);
            if (image == null) {
                return null;
            }
            int largeur = image.getWidth();
            int hauteur = image.getHeight();
            int[] pixels = new int[2 + largeur * hauteur];
            pixels[0] = largeur;
            pixels[1] = hauteur;
            image.getPixels(pixels, 2, largeur, 0, 0, largeur, hauteur);
            image.recycle();
            return pixels;
        } catch (Exception | OutOfMemoryError e) {
            Log.w(JOURNAL, "video : pas d'image fixe : " + e);
            return null;
        } finally {
            relacher(lu);
        }
    }

    /**
     * **Attend une image** au plus `delai` millisecondes, et la tient jusqu'à l'attente suivante
     * (ou la fermeture) : `IMAGE`, `RIEN`, `AU_BOUT` ou `FAUTE`.
     */
    int attendre(long delai) {
        lacher();
        long fin = SystemClock.uptimeMillis() + delai;
        synchronized (signal) {
            while (true) {
                if (faute) {
                    return FAUTE;
                }
                Image image;
                try {
                    image = images.acquireLatestImage();
                } catch (RuntimeException e) {
                    Log.w(JOURNAL, "video : une image ne se lit pas : " + e);
                    return FAUTE;
                }
                if (image != null) {
                    tenue = image;
                    return IMAGE;
                }
                if (auBout) {
                    return AU_BOUT;
                }
                long reste = fin - SystemClock.uptimeMillis();
                if (reste <= 0) {
                    return RIEN;
                }
                try {
                    signal.wait(reste);
                } catch (InterruptedException e) {
                    return RIEN;
                }
            }
        }
    }

    /**
     * **L'image tenue** : ses nombres (pour chaque plan, le pas d'une rangée puis d'un
     * échantillon ; le recadrage x, y, largeur, hauteur ; l'espace de couleurs ; les quarts de
     * tour — `MESURES`, côté Rust), puis ses trois plans.
     */
    Object[] tenue() {
        Image.Plane[] plans = tenue.getPlanes();
        Rect r = tenue.getCropRect();
        int espace = Build.VERSION.SDK_INT >= 33 ? tenue.getDataSpace() : 0;
        int[] mesures = {
            plans[0].getRowStride(), plans[0].getPixelStride(),
            plans[1].getRowStride(), plans[1].getPixelStride(),
            plans[2].getRowStride(), plans[2].getPixelStride(),
            r.left, r.top, r.width(), r.height(), espace, quarts,
        };
        return new Object[] {
            mesures, plans[0].getBuffer(), plans[1].getBuffer(), plans[2].getBuffer()
        };
    }

    /** Rend l'image tenue au lecteur. */
    private void lacher() {
        if (tenue != null) {
            tenue.close();
            tenue = null;
        }
    }

    void jouer(boolean oui) {
        try {
            if (oui) {
                joueur.start();
            } else {
                joueur.pause();
            }
        } catch (IllegalStateException e) {
            Log.w(JOURNAL, "video : " + e);
        }
    }

    void couperLeSon(boolean oui) {
        float volume = oui ? 0f : 1f;
        joueur.setVolume(volume, volume);
    }

    /** Repart du début, et joue. */
    boolean revenirAuDebut() {
        try {
            synchronized (signal) {
                auBout = false;
            }
            joueur.seekTo(0);
            joueur.start();
            return true;
        } catch (IllegalStateException e) {
            return false;
        }
    }

    /** Ferme le lecteur : appelé par Rust quand il le lâche. */
    void fermer() {
        lacher();
        joueur.release();
        images.close();
    }
}
