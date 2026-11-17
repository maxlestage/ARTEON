#!/usr/bin/env python3
"""Les collections du monde sur le site exporté : la page /monde/ et ses données.

  python3 outils/monde_web.py <collections> <vignettes> <site>

  <collections>  le dossier écrit par outils/collections_monde.py (collections.json)
  <vignettes>    le dossier des vignettes du cache de la moisson (<œuvre>.jpg)
  <site>         l'export du site (dist/)

Écrit dans <site>/monde/ :
  index.html            la page, faite de l'accueil exporté (même en-tête, même pied, même
                        style), avec app.js et app.css (outils/monde/)
  donnees/index.json    les musées, les catégories, les siècles, les identifiants
  donnees/liste.json    une ligne par œuvre, pour parcourir et chercher (titre, artiste,
                        date, année, catégorie, musée)
  donnees/fiches-NNN.json  le détail, par 1 000 œuvres (technique, texte, liens…)
  v/<œuvre>.jpg         les vignettes (320 px) ; l'image en grand vient du musée

Il ajoute aussi « Collections » au menu de toutes les pages et une entrée sur l'accueil.
Bibliothèque standard de Python.
"""

import json
import re
import shutil
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
PAR_FICHIER = 1000


def rang(y):
    if y <= -100000:
        return -999999
    return -((-y - 1) // 100 + 1) if y < 0 else (y - 1) // 100 + 1


def romain(n):
    table = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"),
             (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")]
    out = ""
    for v, l in table:
        while n >= v:
            out += l
            n -= v
    return out


def titre_siecle(r):
    if r == -999999:
        return "Avant l'histoire"
    n = abs(r)
    return f"{romain(n)}{'ᵉʳ' if n == 1 else 'ᵉ'} siècle" + (" av. J.-C." if r < 0 else "")


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    collections, vignettes, site = (Path(a) for a in sys.argv[1:])
    donnees = json.loads((collections / "collections.json").read_text(encoding="utf-8"))
    oeuvres = donnees["oeuvres"]
    musees = donnees["musees"]
    position = {m["cle"]: i for i, m in enumerate(musees)}

    sortie = site / "monde"
    shutil.rmtree(sortie, ignore_errors=True)
    (sortie / "donnees").mkdir(parents=True)
    (sortie / "v").mkdir()

    # Les vignettes : seules les œuvres qui en ont une sont gardées.
    gardees = []
    for o in oeuvres:
        source = vignettes / f"{o['i']}.jpg"
        if source.exists():
            shutil.copyfile(source, sortie / "v" / f"{o['i']}.jpg")
            gardees.append(o)

    liste = [[o["t"], o.get("a", ""), o.get("d", ""), o.get("y"), o.get("c", "Autres objets"),
              position[o["k"]], o.get("r")] for o in gardees]
    categories, siecles = {}, {}
    for o in gardees:
        categories[o.get("c", "Autres objets")] = categories.get(o.get("c", "Autres objets"), 0) + 1
        if o.get("y") is not None:
            r = rang(o["y"])
            siecles[r] = siecles.get(r, 0) + 1
    index = {
        "date": donnees.get("date"),
        "total": len(gardees),
        "parFichier": PAR_FICHIER,
        "musees": [{"cle": m["cle"], "nom": m["nom"], "ville": m["ville"], "couleur": m.get("couleur", "#1d3f8f"),
                    "licence": m.get("licence", ""), "oeuvres": sum(1 for o in gardees if o["k"] == m["cle"])}
                   for m in musees],
        "categories": sorted(categories.items(), key=lambda kv: -kv[1]),
        "siecles": [[r, titre_siecle(r), n] for r, n in sorted(siecles.items())],
        "ids": [o["i"] for o in gardees],
    }
    compact = {"ensure_ascii": False, "separators": (",", ":")}
    (sortie / "donnees" / "index.json").write_text(json.dumps(index, **compact), encoding="utf-8")
    (sortie / "donnees" / "liste.json").write_text(json.dumps(liste, **compact), encoding="utf-8")
    for n in range(0, len(gardees), PAR_FICHIER):
        fiches = [{k: o[k] for k in ("v", "m", "s", "o", "x", "u", "w", "g") if o.get(k)}
                  for o in gardees[n:n + PAR_FICHIER]]
        (sortie / "donnees" / f"fiches-{n // PAR_FICHIER:03d}.json").write_text(
            json.dumps(fiches, **compact), encoding="utf-8")
    shutil.copyfile(ICI / "monde" / "app.js", sortie / "app.js")
    shutil.copyfile(ICI / "monde" / "app.css", sortie / "app.css")

    # La page : l'accueil exporté, dont on garde l'en-tête et le pied.
    accueil = (site / "index.html").read_text(encoding="utf-8")
    base = re.search(r'<a class="logo" href="([^"]*)/"', accueil).group(1)
    debut, fin = accueil.index('<main class="page">'), accueil.index("</main>") + len("</main>")
    page = (accueil[:debut]
            + '<main class="page"><div class="monde" id="monde"><noscript>Les collections du monde '
              "demandent JavaScript.</noscript></div></main>"
            + accueil[fin:])
    page = re.sub(r"<title>[^<]*</title>", "<title>Les collections du monde · ARTEON</title>", page, count=1)
    page = page.replace(' class="current"', "").replace(' aria-current="page"', "")
    page = page.replace("</head>", '<link rel="stylesheet" href="app.css"/></head>', 1)
    page = page.replace("</body>", '<script type="module" src="app.js"></script></body>', 1)
    (sortie / "index.html").write_text(page, encoding="utf-8")

    # « Collections » dans le menu de toutes les pages, et une entrée sur l'accueil.
    lien = f'<a href="{base}/monde/">Collections</a>'
    for fichier in site.rglob("*.html"):
        texte = fichier.read_text(encoding="utf-8")
        if lien in texte or "</details><a href=" not in texte:
            continue
        texte = texte.replace("</details><a href=", f"</details>{lien}<a href=", 1)
        fichier.write_text(texte, encoding="utf-8")
    accueil = (site / "index.html").read_text(encoding="utf-8")
    entree = (f'<a class="monde-accueil" href="{base}/monde/"><div><strong>Les collections du monde</strong>'
              f'<span>{f"{len(gardees):,}".replace(",", "\u202f")} œuvres du domaine public de {len(musees)} grands musées : '
              "Chicago, Cleveland, New York, Washington. Chercher, parcourir, regarder en grand.</span></div></a>")
    if "monde-accueil" not in accueil and '<section class="exhibitions ephemeral"' in accueil:
        accueil = accueil.replace('<section class="exhibitions ephemeral"', entree + '<section class="exhibitions ephemeral"', 1)
        accueil = accueil.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
        (site / "index.html").write_text(accueil, encoding="utf-8")

    taille = sum(f.stat().st_size for f in sortie.rglob("*") if f.is_file())
    print(f"✓ /monde/ : {len(gardees)} œuvres, {taille / 1e6:.0f} Mo")


if __name__ == "__main__":
    main()
