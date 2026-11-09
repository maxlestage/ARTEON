#!/usr/bin/env python3
"""Les icônes de l'application ARTEON (PWA), dessinées comme le logo du site : un soleil
et une petite étoile sur un ciel bleu nuit (voir `.logo-mark` dans src/style.rs).

    python3 outils/icones.py     # écrit public/icons/*.png (demande Pillow)

- icon-192.png, icon-512.png : coins arrondis, fond transparent autour ;
- icon-maskable-512.png : tout le carré peint, le dessin dans la zone sûre (80 %) ;
- apple-touch-icon.png (180) : carré plein, iOS arrondit lui-même.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

RACINE = Path(__file__).resolve().parent.parent
SORTIE = RACINE / "public" / "icons"

BLEU_CLAIR = (61, 99, 201)
BLEU = (29, 63, 143)
NUIT = (15, 26, 58)
SOLEIL = (242, 183, 5)
ETOILE = (255, 246, 201)


def melange(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def ciel(taille):
    """Le dégradé radial du logo : clair en haut à droite, nuit dans les coins."""
    image = Image.new("RGB", (taille, taille))
    pixels = image.load()
    cx, cy = 0.7 * taille, 0.3 * taille
    rayon = (taille**2 + taille**2) ** 0.5 * 0.75
    for y in range(taille):
        for x in range(taille):
            t = min(1.0, ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5 / rayon)
            pixels[x, y] = melange(BLEU_CLAIR, BLEU, t / 0.55) if t < 0.55 else melange(BLEU, NUIT, (t - 0.55) / 0.45)
    return image


def dessin(taille, marge=0.0):
    """Le ciel, le soleil et son halo, l'étoile ; `marge` resserre le dessin au centre."""
    image = ciel(taille).convert("RGBA")
    zone = taille * (1 - 2 * marge)
    origine = taille * marge

    def boite(gauche, haut, cote):
        x, y = origine + gauche * zone, origine + haut * zone
        return [x, y, x + cote * zone, y + cote * zone]

    halo = Image.new("RGBA", (taille, taille), (0, 0, 0, 0))
    d = ImageDraw.Draw(halo)
    for etendue, alpha in ((0.13, 38), (0.08, 70), (0.04, 105)):
        b = boite(0.18 - etendue, 0.18 - etendue, 0.46 + 2 * etendue)
        d.ellipse(b, fill=SOLEIL + (alpha,))
    halo = halo.filter(ImageFilter.GaussianBlur(taille * 0.012))
    image = Image.alpha_composite(image, halo)
    d = ImageDraw.Draw(image)
    d.ellipse(boite(0.18, 0.18, 0.46), fill=SOLEIL)
    d.ellipse(boite(1 - 0.14 - 0.18, 1 - 0.16 - 0.18, 0.18), fill=ETOILE)
    return image


def arrondi(image, rayon):
    masque = Image.new("L", image.size, 0)
    ImageDraw.Draw(masque).rounded_rectangle([0, 0, image.size[0] - 1, image.size[1] - 1], rayon, fill=255)
    image.putalpha(masque)
    return image


def main():
    SORTIE.mkdir(parents=True, exist_ok=True)
    grand = 1024  # dessiné en grand, puis réduit : des bords nets
    for nom, taille in (("icon-192.png", 192), ("icon-512.png", 512)):
        icone = arrondi(dessin(grand), grand * 7 // 28).resize((taille, taille), Image.LANCZOS)
        icone.save(SORTIE / nom, optimize=True)
    dessin(grand, marge=0.12).resize((512, 512), Image.LANCZOS).save(SORTIE / "icon-maskable-512.png", optimize=True)
    dessin(grand).convert("RGB").resize((180, 180), Image.LANCZOS).save(SORTIE / "apple-touch-icon.png", optimize=True)
    arrondi(dessin(grand), grand * 7 // 28).resize((32, 32), Image.LANCZOS).save(SORTIE / "favicon-32.png", optimize=True)
    print("icônes écrites dans", SORTIE)


if __name__ == "__main__":
    main()
