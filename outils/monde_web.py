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
import unicodedata
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


ALIAS = json.loads((ICI / "monde" / "alias.json").read_text(encoding="utf-8"))


def plier(texte):
    texte = str(texte or "").replace("œ", "oe").replace("Œ", "oe").replace("æ", "ae").replace("Æ", "ae")
    return "".join(c for c in unicodedata.normalize("NFD", texte) if not unicodedata.combining(c)).lower()


def cle_nom(nom):
    """La clé d'un artiste, la même que dans Explorer (explorer.js, cleNom) et dans l'app :
    « Rembrandt (Rembrandt van Rijn) », « Rembrandt van Rijn » et « Rembrandt » ne font qu'un."""
    n = str(nom or "").strip()
    m = re.match(r"^(.+?)\s*\((.+)\)$", n)
    if m:
        premier = (plier(m.group(1)).split() or [""])[0]
        n = m.group(2) if premier in plier(m.group(2)) else m.group(1)
    k = " ".join(re.sub(r"[-‐–'’.,]", " ", plier(n)).split())
    return ALIAS.get(k, k)


LIENS = json.loads((ICI / "monde" / "liens_expositions.json").read_text(encoding="utf-8"))


def oeuvres_liees(catalogue, gardees):
    """Pour chaque exposition, toutes les œuvres des collections du monde qui s'y rattachent
    (outils/monde/liens_expositions.json) : celles de ses artistes, et celles qui
    répondent à ses conditions (catégories, mots, origine, années). De la plus ancienne à la
    plus récente."""
    inconnu = cle_nom("Artiste inconnu")
    cles = [cle_nom(o.get("a")) for o in gardees]
    resultat = {}
    for ex in catalogue["exhibitions"]:
        if ex.get("ephemeral"):
            continue
        r = LIENS.get(ex["slug"], {})
        noms = {cle_nom(w.get("artist") or ex.get("artist")) for w in ex["works"]}
        noms |= {cle_nom(a) for a in r.get("artistes", [])}
        noms -= {inconnu, ""}
        categories = set(r.get("categories", []))
        mots = re.compile(r["mots"], re.I) if r.get("mots") else None
        origine = re.compile(r["origine"], re.I) if r.get("origine") else None
        conditions = bool(categories or mots or origine)
        de, a = r.get("de", -10**9), r.get("a", 10**9)
        indices = []
        for i, o in enumerate(gardees):
            y = o.get("y")
            if y is not None and not de <= y <= a:
                continue
            if cles[i] in noms and not r.get("seulement_conditions"):
                indices.append(i)
                continue
            if not conditions:
                continue
            if categories and o.get("c", "Autres objets") not in categories:
                continue
            texte = o.get("t", "") if r.get("dans_titre") else f"{o.get('t', '')} {o.get('m', '')} {o.get('o', '')}"
            if mots and not mots.search(texte):
                continue
            if origine and not origine.search(f"{o.get('o', '')} {o.get('v', '')} {o.get('a', '')}"):
                continue
            indices.append(i)
        indices.sort(key=lambda i: (gardees[i].get("y") is None, gardees[i].get("y") or 0, gardees[i].get("t", "")))
        resultat[ex["slug"]] = {"nom": ex.get("short_name") or ex["title"], "titre": ex["title"], "indices": indices}
    return resultat


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
    for fichier in sorted((ICI / "monde").iterdir()):
        if fichier.suffix in (".js", ".css", ".json"):
            shutil.copyfile(fichier, sortie / fichier.name)

    # Les pages : l'accueil exporté, dont on garde l'en-tête, le style et le pied.
    accueil = (site / "index.html").read_text(encoding="utf-8")
    base = re.search(r'<a class="logo" href="([^"]*)/"', accueil).group(1)
    debut, fin = accueil.index('<main class="page">'), accueil.index("</main>") + len("</main>")

    def ecrire(dossier, titre, script):
        page = (accueil[:debut]
                + f'<main class="page"><div class="monde" id="monde"><noscript>{titre} demande JavaScript.'
                  "</noscript></div></main>"
                + accueil[fin:])
        page = re.sub(r"<title>[^<]*</title>", f"<title>{titre} · ARTEON</title>", page, count=1)
        page = page.replace(' class="current"', "").replace(' aria-current="page"', "")
        page = page.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
        page = page.replace("</body>", f'<script type="module" src="{base}/monde/{script}"></script></body>', 1)
        (site / dossier).mkdir(parents=True, exist_ok=True)
        (site / dossier / "index.html").write_text(page, encoding="utf-8")

    ecrire("monde", "Les collections du monde", "app.js")
    ecrire("musees", "Les musées", "musees.js")
    ecrire("explorer", "Explorer", "explorer.js")

    # « Musées », « Collections » et « Explorer » dans le menu de toutes les pages.
    liens = (f'<a href="{base}/musees/">Musées</a><a href="{base}/monde/">Collections</a>'
             f'<a href="{base}/explorer/">Explorer</a>')
    for fichier in site.rglob("*.html"):
        texte = fichier.read_text(encoding="utf-8")
        if liens in texte or "</details><a href=" not in texte:
            continue
        texte = texte.replace("</details><a href=", f"</details>{liens}<a href=", 1)
        fichier.write_text(texte, encoding="utf-8")

    # L'accueil : le vrai nombre d'œuvres, et l'entrée des musées, des collections, d'Explorer.
    accueil = (site / "index.html").read_text(encoding="utf-8")
    fine = lambda n: f"{n:,}".replace(",", "\u202f")
    entree = (f'<div class="monde-entrees">'
              f'<a class="monde-accueil" href="{base}/monde/"><div><strong>Les collections du monde</strong>'
              f'<span>{fine(len(gardees))} œuvres du domaine public de {len(musees)} grands musées : '
              "Chicago, Cleveland, New York, Washington. Chercher, parcourir, regarder en grand.</span></div></a>"
              f'<a class="monde-accueil musees" href="{base}/musees/"><div><strong>Les musées</strong>'
              "<span>Les vrais musées reproduits dans ARTEON, leurs plans salle par salle, et le Grand Musée ARTEON.</span></div></a>"
              f'<a class="monde-accueil explorer" href="{base}/explorer/"><div><strong>Explorer</strong>'
              "<span>Les artistes de A à Z, la chronologie, les thèmes, et le jeu « Qui l'a fait ? ».</span></div></a>"
              "</div>")
    if "monde-entrees" not in accueil and '<section class="exhibitions ephemeral"' in accueil:
        accueil = accueil.replace('<section class="exhibitions ephemeral"', entree + '<section class="exhibitions ephemeral"', 1)
        accueil = accueil.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
    (site / "index.html").write_text(accueil, encoding="utf-8")

    # Les mêmes nombres partout : le pied de page de chaque page et l'accueil comptent
    # les œuvres des expositions ET des collections du monde, et tous leurs artistes.
    catalogue = json.loads((site / "catalogue.json").read_text(encoding="utf-8"))
    noms = {w.get("artist") or ex.get("artist") for ex in catalogue["exhibitions"] for w in ex["works"]}
    noms |= {o.get("a") for o in gardees}
    noms -= {None, "", "Artiste inconnu"}
    artistes = len({cle_nom(n) for n in noms})
    oeuvres_expositions = sum(len(ex["works"]) for ex in catalogue["exhibitions"])
    oeuvres = oeuvres_expositions + len(gardees)
    chiffre = r"\d[\d\u202f\u00a0 ]*"
    for fichier in site.rglob("*.html"):
        texte = fichier.read_text(encoding="utf-8")
        neuf = re.sub(rf"(class=\"footer-figures\">{chiffre} expositions · ){chiffre} œuvres · {chiffre} artistes",
                      rf"\g<1>{fine(oeuvres)} œuvres · {fine(artistes)} artistes", texte)
        neuf = re.sub(rf"{chiffre} artistes et ateliers : ", f"{fine(artistes)} artistes et ateliers : ", neuf)
        neuf = re.sub(rf"<strong>{chiffre}</strong><span>œuvres</span>",
                      f"<strong>{fine(oeuvres)}</strong><span>œuvres</span>", neuf)
        neuf = re.sub(rf"<strong>{chiffre}</strong><span>artistes</span>",
                      f"<strong>{fine(artistes)}</strong><span>artistes</span>", neuf)
        if neuf != texte:
            fichier.write_text(neuf, encoding="utf-8")
    print(f"✓ partout : {fine(oeuvres)} œuvres ({oeuvres_expositions} dans les expositions), {fine(artistes)} artistes")

    # Chaque exposition : toutes ses œuvres liées dans les collections du monde, sur une page
    # (/monde/#/exposition/<nom>), annoncée en tête de la page de l'exposition.
    liees = oeuvres_liees(catalogue, gardees)
    (sortie / "donnees" / "expositions.json").write_text(json.dumps(liees, **compact), encoding="utf-8")
    for slug, info in liees.items():
        page = site / slug / "index.html"
        n = len(info["indices"])
        if not n or not page.exists():
            continue
        texte = page.read_text(encoding="utf-8")
        if "monde-liees" in texte or '<section class="building"' not in texte:
            continue
        propres = sum(len(ex["works"]) for ex in catalogue["exhibitions"] if ex["slug"] == slug)
        bloc = (f'<section class="monde-liees"><a class="monde-accueil" href="{base}/monde/#/exposition/{slug}"><div>'
                f'<strong>Toutes les œuvres de l\'exposition : {fine(propres + n)}</strong>'
                f'<span>Les {propres} œuvres de la visite, et {fine(n)} œuvres liées dans les collections du monde, '
                f'de la plus ancienne à la plus récente →</span></div></a></section>')
        texte = texte.replace('<section class="building"', bloc + '<section class="building"', 1)
        if "monde/app.css" not in texte:
            texte = texte.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
        page.write_text(texte, encoding="utf-8")
    print(f"✓ expositions : {sum(len(v['indices']) for v in liees.values())} liens vers les collections du monde")

    taille = sum(f.stat().st_size for f in sortie.rglob("*") if f.is_file())
    print(f"✓ /monde/ : {len(gardees)} œuvres, {taille / 1e6:.0f} Mo")


if __name__ == "__main__":
    main()
