"""Un pavé tactile virtuel (uinput), que libinput — donc niri — traite comme le vrai (fiche 112).

Par `essai_ecran.sh`. Son compte a le droit d'écrire sur `/dev/uinput` (une ACL de logind) ; le
pavé n'existe que le temps du programme.

    python3 pave_virtuel.py <ordres.txt>

Le pavé imite celui du VivoBook (ELAN1200 04F3:303E : I2C, « buttonpad », cinq doigts) ; ses plages
exactes ne se lisent pas sans le groupe `input` : 3209 × 2097 unités à 31 par mm (≈ 104 × 68 mm),
plausibles pour ce pavé. Chaque ordre, une ligne :
  pincer <facteur> <ms>     deux doigts qui s'écartent (facteur > 1 : zoomer) ou se rapprochent
  glisser <dx_mm> <dy_mm> <ms>   deux doigts qui glissent ensemble (le défilement à deux doigts)
  attendre <ms>
Les événements partent à 125 Hz, comme un pavé I2C ; le temps est celui du noyau, à l'écriture.
"""
import sys
import time
from evdev import UInput, AbsInfo, ecodes as e

LX, LY, RES = 3209, 2097, 31
HZ = 125
caps = {
    e.EV_KEY: [e.BTN_LEFT, e.BTN_TOOL_FINGER, e.BTN_TOOL_DOUBLETAP, e.BTN_TOOL_TRIPLETAP,
               e.BTN_TOOL_QUADTAP, e.BTN_TOOL_QUINTTAP, e.BTN_TOUCH],
    e.EV_ABS: [
        (e.ABS_X, AbsInfo(0, 0, LX, 0, 0, RES)),
        (e.ABS_Y, AbsInfo(0, 0, LY, 0, 0, RES)),
        (e.ABS_MT_SLOT, AbsInfo(0, 0, 4, 0, 0, 0)),
        (e.ABS_MT_POSITION_X, AbsInfo(0, 0, LX, 0, 0, RES)),
        (e.ABS_MT_POSITION_Y, AbsInfo(0, 0, LY, 0, 0, RES)),
        (e.ABS_MT_TRACKING_ID, AbsInfo(0, 0, 65535, 0, 0, 0)),
    ],
}
ui = UInput(caps, name="Glucose banc pave tactile", vendor=0x04F3, product=0x303E, version=0x100,
            bustype=e.BUS_I2C, input_props=[e.INPUT_PROP_POINTER, e.INPUT_PROP_BUTTONPAD])
time.sleep(1.5)  # le temps que niri l'ajoute
suivi = [100]


def poser(doigts, bas):
    """doigts : [(x, y)] en unités ; bas : le geste commence."""
    for i, (x, y) in enumerate(doigts):
        ui.write(e.EV_ABS, e.ABS_MT_SLOT, i)
        if bas:
            suivi[0] += 1
            ui.write(e.EV_ABS, e.ABS_MT_TRACKING_ID, suivi[0])
        ui.write(e.EV_ABS, e.ABS_MT_POSITION_X, int(x))
        ui.write(e.EV_ABS, e.ABS_MT_POSITION_Y, int(y))
    if bas:
        ui.write(e.EV_KEY, e.BTN_TOUCH, 1)
        ui.write(e.EV_KEY, e.BTN_TOOL_DOUBLETAP, 1)
    ui.write(e.EV_ABS, e.ABS_X, int(doigts[0][0]))
    ui.write(e.EV_ABS, e.ABS_Y, int(doigts[0][1]))
    ui.syn()


def lever(n):
    for i in range(n):
        ui.write(e.EV_ABS, e.ABS_MT_SLOT, i)
        ui.write(e.EV_ABS, e.ABS_MT_TRACKING_ID, -1)
    ui.write(e.EV_KEY, e.BTN_TOUCH, 0)
    ui.write(e.EV_KEY, e.BTN_TOOL_DOUBLETAP, 0)
    ui.syn()


def geste(position, ms):
    """position(t), t de 0 à 1 → deux doigts ; à HZ, en temps réel."""
    n = max(2, int(ms * HZ / 1000))
    debut = time.monotonic()
    poser(position(0.0), True)
    for k in range(1, n + 1):
        cible = debut + k / HZ
        time.sleep(max(0.0, cible - time.monotonic()))
        poser(position(k / n), False)
    time.sleep(1 / HZ)
    lever(2)


cx, cy = LX / 2, LY * 0.45
for ligne in open(sys.argv[1]):
    m = ligne.split()
    if not m or m[0].startswith('#'):
        continue
    if m[0] == 'pincer':
        f, ms = float(m[1]), float(m[2])
        d0 = 18 * RES  # 18 mm entre les doigts au départ
        d1 = d0 * f

        def position(t, d0=d0, d1=d1):
            d = d0 + (d1 - d0) * t
            return [(cx - d / 2, cy), (cx + d / 2, cy)]
        geste(position, ms)
    elif m[0] == 'glisser':
        dx, dy, ms = float(m[1]) * RES, float(m[2]) * RES, float(m[3])
        x0, y0 = cx - dx / 2, cy - dy / 2

        def position(t, x0=x0, y0=y0, dx=dx, dy=dy):
            return [(x0 - 9 * RES + dx * t, y0 + dy * t), (x0 + 9 * RES + dx * t, y0 + dy * t)]
        geste(position, ms)
    elif m[0] == 'attendre':
        time.sleep(float(m[1]) / 1000)
    print('fait', ligne.strip(), flush=True)
ui.close()
