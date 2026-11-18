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
  v/<œuvre>.webp        les vignettes (320 px, WebP) ; l'image en grand vient du musée

Il ajoute aussi « Collections » au menu de toutes les pages et une entrée sur l'accueil.
Python et Pillow (pour les vignettes en WebP).
"""

import concurrent.futures
import html
import json
import re
import shutil
import sys
import time
import unicodedata
from pathlib import Path

from PIL import Image

ICI = Path(__file__).resolve().parent
PAR_FICHIER = 1000
# Les vignettes hébergées sur le site (WebP, ~6 Ko) : au plus tant, pour que le site reste
# sous la limite de GitHub Pages (1 Go). Au-delà, la vignette vient du musée.
MAX_HEBERGEES = 120_000


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


def en_webp(chemins):
    """Une vignette JPEG du cache, réécrite en WebP (qualité 50 : ~6 Ko au lieu de ~10)."""
    source, cible = chemins
    with Image.open(source) as image:
        image.convert("RGB").save(cible, "WEBP", quality=50, method=6)


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

    # Les vignettes : seules les œuvres qui en ont une sont gardées, en WebP (deux fois plus
    # léger que le JPEG : toutes les œuvres tiennent sur le site).
    gardees = [o for o in oeuvres if (vignettes / f"{o['i']}.jpg").exists()]

    # La réserve : toutes les autres œuvres libres trouvées par la moisson. Leur vignette est
    # hébergée quand la moisson l'a téléchargée ; sinon elle vient du musée (« vu »).
    fichier_reserve = collections / "reserve.json"
    if fichier_reserve.exists():
        vus = {o["i"] for o in gardees}
        for o in json.loads(fichier_reserve.read_text(encoding="utf-8")):
            if o.get("k") not in position or o["i"] in vus:
                continue
            local = (vignettes / f"{o['i']}.jpg").exists()
            if local and len(gardees) < MAX_HEBERGEES:
                o.pop("vu", None)
            elif not o.get("vu"):
                continue  # ni vignette hébergée, ni vignette du musée
            vus.add(o["i"])
            gardees.append(o)
    a_convertir = [o["i"] for o in gardees if not o.get("vu")]
    hebergees = len(a_convertir)
    with concurrent.futures.ProcessPoolExecutor() as pool:
        list(pool.map(en_webp, [(vignettes / f"{i}.jpg", sortie / "v" / f"{i}.webp") for i in a_convertir],
                      chunksize=500))

    liste = [[o["t"], o.get("a", ""), o.get("d", ""), o.get("y"), o.get("c", "Autres objets"),
              position[o["k"]], o.get("r")] + ([o["vu"]] if o.get("vu") else []) for o in gardees]
    categories, siecles = {}, {}
    for o in gardees:
        categories[o.get("c", "Autres objets")] = categories.get(o.get("c", "Autres objets"), 0) + 1
        if o.get("y") is not None:
            r = rang(o["y"])
            siecles[r] = siecles.get(r, 0) + 1
    index = {
        "date": donnees.get("date"),
        "total": len(gardees),
        "hebergees": hebergees,
        "parFichier": PAR_FICHIER,
        "musees": [{"cle": m["cle"], "nom": m["nom"], "ville": m["ville"], "couleur": m.get("couleur", "#1d3f8f"),
                    "licence": m.get("licence", ""), "oeuvres": sum(1 for o in gardees if o["k"] == m["cle"])}
                   for m in musees],
        "categories": sorted(categories.items(), key=lambda kv: -kv[1]),
        "siecles": [[r, titre_siecle(r), n] for r, n in sorted(siecles.items())],
        "ids": [o["i"] for o in gardees],
    }
    compact = {"ensure_ascii": False, "separators": (",", ":")}
    # La version de cette mise en ligne : les pages la demandent toujours au réseau et s'en
    # servent pour ne jamais mélanger les fichiers de deux mises en ligne (memoire.js).
    (sortie / "donnees" / "version.json").write_text(json.dumps({"v": f"{int(time.time()):x}"}), encoding="utf-8")
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

    # Les pages, dans chaque langue (/, /en/, /es/) : l'accueil exporté de cette langue,
    # dont on garde l'en-tête (le menu), le style et le pied.
    LANGUES = {
        "fr": {"prefixe": "", "js": "demande JavaScript.", "monde": "Les collections du monde", "musees": "Les musées", "explorer": "Explorer"},
        "en": {"prefixe": "/en", "js": "needs JavaScript.", "monde": "World collections", "musees": "Museums", "explorer": "Explore"},
        "es": {"prefixe": "/es", "js": "necesita JavaScript.", "monde": "Colecciones del mundo", "musees": "Museos", "explorer": "Explorar"},
    }
    fine = lambda n: f"{n:,}".replace(",", " ")
    accueil_fr = (site / "index.html").read_text(encoding="utf-8")
    base = re.search(r'<a class="logo" href="([^"]*)/"', accueil_fr).group(1)
    for code, L in LANGUES.items():
        dossier_langue = site / L["prefixe"].lstrip("/") if L["prefixe"] else site
        if not (dossier_langue / "index.html").exists():
            continue
        accueil = (dossier_langue / "index.html").read_text(encoding="utf-8")
        debut, fin = accueil.index('<main class="page">'), accueil.index("</main>") + len("</main>")

        def ecrire(dossier, titre, script):
            page = (accueil[:debut]
                    + f'<main class="page"><div class="monde" id="monde"><noscript>{titre} {L["js"]}'
                      "</noscript></div></main>"
                    + accueil[fin:])
            page = re.sub(r"<title>[^<]*</title>", f"<title>{titre} · ARTEON</title>", page, count=1)
            page = page.replace(' class="current"', "").replace(' aria-current="page"', "")
            # Les liens vers les autres langues : cette page-ci, pas l'accueil.
            for autre in LANGUES.values():
                racine = f'{base}{autre["prefixe"]}/'
                page = page.replace(f'hreflang="{[k for k, v in LANGUES.items() if v is autre][0]}" href="{racine}"',
                                    f'hreflang="{[k for k, v in LANGUES.items() if v is autre][0]}" href="{racine}{dossier}/"')
                page = page.replace(f'href="{racine}" hreflang=', f'href="{racine}{dossier}/" hreflang=')
            page = page.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
            page = page.replace("</body>", f'<script type="module" src="{base}/monde/{script}"></script></body>', 1)
            (dossier_langue / dossier).mkdir(parents=True, exist_ok=True)
            (dossier_langue / dossier / "index.html").write_text(page, encoding="utf-8")

        ecrire("monde", L["monde"], "app.js")
        ecrire("musees", L["musees"], "musees.js")
        ecrire("explorer", L["explorer"], "explorer.js")

        # L'accueil : l'entrée des musées, des collections et d'Explorer.
        p = f'{base}{L["prefixe"]}'
        textes = {
            "fr": (f"Les collections du monde", f"{fine(len(gardees))} œuvres du domaine public de {len(musees)} grands musées : "
                   "Chicago, Cleveland, New York, Washington. Chercher, parcourir, regarder en grand.",
                   "Les musées", "Les vrais musées reproduits dans ARTEON, leurs plans salle par salle, et le Grand Musée ARTEON.",
                   "Explorer", "Les artistes de A à Z, la chronologie, les thèmes, et le jeu « Qui l'a fait ? »."),
            "en": ("World collections", f"{fine(len(gardees))} public-domain works from {len(musees)} great museums: "
                   "Chicago, Cleveland, New York, Washington. Search, browse, view them large.",
                   "Museums", "The real museums reproduced in ARTEON, their plans room by room, and the Grand ARTEON Museum.",
                   "Explore", "Artists from A to Z, the timeline, themes, and the “Who made it?” game."),
            "es": ("Colecciones del mundo", f"{fine(len(gardees))} obras de dominio público de {len(musees)} grandes museos: "
                   "Chicago, Cleveland, Nueva York, Washington. Buscar, recorrer, ver en grande.",
                   "Museos", "Los museos reales reproducidos en ARTEON, sus planos sala por sala y el Gran Museo ARTEON.",
                   "Explorar", "Los artistas de la A a la Z, la cronología, los temas y el juego «¿Quién lo hizo?»."),
        }[code]
        entree = (f'<div class="monde-entrees">'
                  f'<a class="monde-accueil" href="{p}/monde/"><div><strong>{textes[0]}</strong><span>{textes[1]}</span></div></a>'
                  f'<a class="monde-accueil musees" href="{p}/musees/"><div><strong>{textes[2]}</strong><span>{textes[3]}</span></div></a>'
                  f'<a class="monde-accueil explorer" href="{p}/explorer/"><div><strong>{textes[4]}</strong><span>{textes[5]}</span></div></a>'
                  "</div>")
        if "monde-entrees" not in accueil and '<section class="exhibitions ephemeral"' in accueil:
            accueil = accueil.replace('<section class="exhibitions ephemeral"', entree + '<section class="exhibitions ephemeral"', 1)
            accueil = accueil.replace("</head>", f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>', 1)
        (dossier_langue / "index.html").write_text(accueil, encoding="utf-8")

    # Les mêmes nombres partout, dans les trois langues : le pied de page de chaque page et
    # l'accueil comptent les œuvres des expositions ET des collections du monde, et tous
    # leurs artistes.
    catalogue = json.loads((site / "catalogue.json").read_text(encoding="utf-8"))
    noms = {w.get("artist") or ex.get("artist") for ex in catalogue["exhibitions"] for w in ex["works"]}
    noms |= {o.get("a") for o in gardees}
    # … et l'annuaire : tous les artistes du domaine public dont les musées montrent une œuvre.
    annuaire = ICI / "monde" / "annuaire.json"
    if annuaire.exists():
        noms |= {a[0] for a in json.loads(annuaire.read_text(encoding="utf-8"))}
    noms -= {None, "", "Artiste inconnu"}
    artistes = len({cle_nom(n) for n in noms})
    oeuvres_expositions = sum(len(ex["works"]) for ex in catalogue["exhibitions"])
    oeuvres = oeuvres_expositions + len(gardees)
    chiffre = r"\d[\d  , ]*"
    mots = [  # (expositions, œuvres, artistes, « artistes et ateliers »)
        ("expositions", "œuvres", "artistes", "artistes et ateliers"),
        ("exhibitions", "works", "artists", "artists and workshops"),
        ("exposiciones", "obras", "artistas", "artistas y talleres"),
    ]
    for fichier in site.rglob("*.html"):
        texte = fichier.read_text(encoding="utf-8")
        neuf = texte
        for ex_, oe, ar, ateliers in mots:
            neuf = re.sub(rf"(class=\"footer-figures\">{chiffre} {ex_} · ){chiffre} {oe} · {chiffre} {ar}",
                          rf"\g<1>{fine(oeuvres)} {oe} · {fine(artistes)} {ar}", neuf)
            neuf = re.sub(rf"{chiffre} {ateliers}: ", f"{fine(artistes)} {ateliers}: ", neuf)
            neuf = re.sub(rf"{chiffre} {ateliers} : ", f"{fine(artistes)} {ateliers} : ", neuf)
            neuf = re.sub(rf"<strong>{chiffre}</strong><span>{oe}</span>",
                          f"<strong>{fine(oeuvres)}</strong><span>{oe}</span>", neuf)
            neuf = re.sub(rf"<strong>{chiffre}</strong><span>{ar}</span>",
                          f"<strong>{fine(artistes)}</strong><span>{ar}</span>", neuf)
        if neuf != texte:
            fichier.write_text(neuf, encoding="utf-8")
    print(f"✓ partout : {fine(oeuvres)} œuvres ({oeuvres_expositions} dans les expositions), {fine(artistes)} artistes")

    # Chaque exposition : toutes ses œuvres liées dans les collections du monde, sur une page
    # (/monde/#/exposition/<nom>), annoncée en tête de la page de l'exposition.
    liees = oeuvres_liees(catalogue, gardees)
    (sortie / "donnees" / "expositions.json").write_text(json.dumps(liees, **compact), encoding="utf-8")
    mots_oeuvres = {"fr": ("œuvres", "en 3D"), "en": ("works", "in 3D"), "es": ("obras", "en 3D")}
    for slug, info in liees.items():
        n = len(info["indices"])
        exposition = next(ex for ex in catalogue["exhibitions"] if ex["slug"] == slug)
        propres = len(exposition["works"])
        # Les artistes de l'exposition : la page « Œuvres » demande à l'API toutes leurs
        # autres œuvres dans les musées du monde (outils/monde/liees.js).
        artistes = []
        for w in exposition["works"]:
            a = w.get("artist") or exposition.get("artist")
            if a and a != "Artiste inconnu" and a not in artistes:
                artistes.append(a)
        artistes_json = html.escape(json.dumps(artistes, ensure_ascii=False), quote=True)
        # Les œuvres liées, prêtes à afficher sur la page « Œuvres » de l'exposition
        # (outils/monde/liees.js) : identifiant, titre, artiste, date, vignette du musée.
        (sortie / "donnees" / f"expo-{slug}.json").write_text(json.dumps(
            [[gardees[i]["i"], gardees[i]["t"], gardees[i].get("a", ""), gardees[i].get("d", "")]
             + ([gardees[i]["vu"]] if gardees[i].get("vu") else []) for i in info["indices"]], **compact), encoding="utf-8")
        for code, L in LANGUES.items():
            dossier_langue = site / L["prefixe"].lstrip("/") if L["prefixe"] else site
            p = f'{base}{L["prefixe"]}'
            titre, phrase, titre_page, phrase_page, total, suite = {
                "fr": ("Toutes les œuvres de l'exposition et de ses artistes",
                       f"Les {propres} œuvres de la visite, {fine(n)} œuvres liées dans les collections du monde, et toutes les œuvres de chaque artiste dans les musées, sur la même page →",
                       f"{fine(n)} œuvres de plus dans les collections du monde",
                       "Les œuvres des mêmes artistes et du même sujet dans les grands musées, de la plus ancienne à la plus récente.",
                       f"Toutes les œuvres : {fine(propres + n)}…",
                       f"Cette page les montre toutes : les {propres} de la visite 3D d'abord, puis celles des collections du monde, puis toutes les œuvres de chaque artiste dans les musées du monde. Elles s'affichent à mesure que l'on descend."),
                "en": ("All the works of the exhibition and its artists",
                       f"The {propres} works on the tour, {fine(n)} related works in the world collections, and every work by each artist in the museums, all on one page →",
                       f"{fine(n)} more works in the world collections",
                       "Works by the same artists and on the same subject in the great museums, from oldest to most recent.",
                       f"All the works: {fine(propres + n)}…",
                       f"This page shows them all: the {propres} on the 3D tour first, then those in the world collections, then every work by each artist in the world's museums. They appear as you scroll down."),
                "es": ("Todas las obras de la exposición y de sus artistas",
                       f"Las {propres} obras de la visita, {fine(n)} obras relacionadas en las colecciones del mundo y todas las obras de cada artista en los museos, en la misma página →",
                       f"{fine(n)} obras más en las colecciones del mundo",
                       "Obras de los mismos artistas y del mismo tema en los grandes museos, de la más antigua a la más reciente.",
                       f"Todas las obras: {fine(propres + n)}…",
                       f"Esta página las muestra todas: primero las {propres} de la visita 3D, luego las de las colecciones del mundo y después todas las obras de cada artista en los museos del mundo. Aparecen a medida que se baja."),
            }[code]
            css = f'<link rel="stylesheet" href="{base}/monde/app.css"/></head>'
            # La présentation : l'annonce, qui mène à la page des œuvres.
            page = dossier_langue / slug / "index.html"
            if page.exists():
                texte = page.read_text(encoding="utf-8")
                if "monde-liees" not in texte and '<section class="building"' in texte:
                    bloc = (f'<section class="monde-liees"><a class="monde-accueil" href="{p}/{slug}/oeuvres/#oeuvres-liees"><div>'
                            f'<strong>{titre}</strong><span>{phrase}</span></div></a></section>')
                    texte = texte.replace('<section class="building"', bloc + '<section class="building"', 1)
                    if "monde/app.css" not in texte:
                        texte = texte.replace("</head>", css, 1)
                    page.write_text(texte, encoding="utf-8")
            # La page des œuvres : toutes, à la suite de celles de la visite.
            page = dossier_langue / slug / "oeuvres" / "index.html"
            if page.exists():
                texte = page.read_text(encoding="utf-8")
                if 'id="oeuvres-liees"' not in texte and "</main>" in texte:
                    bloc = (f'<section class="monde monde-suite" id="oeuvres-liees" data-exposition="{slug}" '
                            f'data-visite="{propres}" data-artistes="{artistes_json}">'
                            f'<section class="monde-liees-collections"><h2 class="section-title">{titre_page}</h2>'
                            f'<p class="lead">{phrase_page}</p><p class="monde-compte"></p><div class="monde-grille"></div></section>'
                            '</section>')
                    texte = texte.replace("</main>", bloc + "</main>", 1)
                    # En tête de page : le nombre de toutes les œuvres (complété par liees.js).
                    annonce = (f'<p class="monde-total"><strong id="total-oeuvres">{total}</strong> {suite} '
                               f'<a href="#oeuvres-liees">↓</a></p>')
                    texte = texte.replace("</h1>", "</h1>" + annonce, 1)
                    if "monde/app.css" not in texte:
                        texte = texte.replace("</head>", css, 1)
                    texte = texte.replace("</body>", f'<script type="module" src="{base}/monde/liees.js"></script></body>', 1)
                    page.write_text(texte, encoding="utf-8")
            # L'accueil : la carte de l'exposition compte toutes ses œuvres.
            accueil = dossier_langue / "index.html"
            if accueil.exists():
                texte = accueil.read_text(encoding="utf-8")
                mot, en3d = mots_oeuvres[code]
                neuf = re.sub(rf'(<article class="ex-card[^>]*id="{re.escape(slug)}".*?<p class="tag">)(\d+) {mot} · ',
                              rf"\g<1>{fine(propres + n)} {mot} · \g<2> {en3d} · ", texte, count=1, flags=re.S)
                if neuf != texte:
                    accueil.write_text(neuf, encoding="utf-8")
    print(f"✓ expositions : {sum(len(v['indices']) for v in liees.values())} liens vers les collections du monde")

    taille = sum(f.stat().st_size for f in sortie.rglob("*") if f.is_file())
    print(f"✓ /monde/ : {len(gardees)} œuvres, {taille / 1e6:.0f} Mo")


if __name__ == "__main__":
    main()
