#!/usr/bin/env python3
"""Les collections du monde : des dizaines de milliers d'œuvres du domaine public, pour
l'application iOS (ios/ARTEON.swiftpm/Musee/collections/).

Le script interroge les collections ouvertes de quatre musées, qui publient en CC0 les
données et les images de leurs œuvres du domaine public :

  - l'Art Institute of Chicago (api.artic.edu) ;
  - le Cleveland Museum of Art (openaccess-api.clevelandart.org) ;
  - le Metropolitan Museum of Art (collectionapi.metmuseum.org) ;
  - la National Gallery of Art de Washington (données ouvertes en CSV, images IIIF).

Il choisit les œuvres (les chefs-d'œuvre signalés par les musées et les peintures
d'abord, puis la sculpture, le dessin, les objets…), garde de chacune une fiche et une
vignette de 320 px, et écrit :

  collections.json          les musées et les fiches (titre, artiste, date, technique,
                            catégorie, origine, texte, lien, image en grand…)
  vignettes-<musée>.bin     les vignettes JPEG bout à bout ; chaque fiche dit où est la
                            sienne (décalage, longueur) : un seul fichier par musée plutôt
                            que des dizaines de milliers.

L'image en grand n'est pas copiée : l'application la télécharge au premier affichage, puis
la garde. Un cache (--cache) garde les fiches une semaine et les vignettes pour toujours :
seules les nouveautés sont téléchargées d'une fois sur l'autre.

  python3 outils/collections_monde.py --sortie ios/ARTEON.swiftpm/Musee/collections
  python3 outils/collections_monde.py --total 400 --sortie /tmp/essai     (un petit essai)

Bibliothèque standard de Python, et Pillow pour réduire les images.
"""

import argparse
import concurrent.futures
import csv
import datetime
import html
import io
import json
import random
import re
import sys
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    sys.exit("Pillow est nécessaire : pip install pillow")

AGENT = "ARTEON/1.0 (https://github.com/maxlestage/ARTEON)"
VIGNETTE = 320
QUALITE = 62
VERSION = 1

# La part de chaque musée dans le total.
PARTS = {"chicago": 0.30, "cleveland": 0.24, "met": 0.26, "washington": 0.20}

MUSEES = {
    "chicago": {
        "nom": "Art Institute of Chicago", "ville": "Chicago, États-Unis",
        "site": "https://www.artic.edu", "couleur": "#b5121b",
        "licence": "Images et données CC0 ; textes de présentation CC BY 4.0 (Art Institute of Chicago)",
    },
    "cleveland": {
        "nom": "Cleveland Museum of Art", "ville": "Cleveland, États-Unis",
        "site": "https://www.clevelandart.org", "couleur": "#1f5c4a",
        "licence": "Images et données CC0 (Cleveland Museum of Art Open Access)",
    },
    "met": {
        "nom": "Metropolitan Museum of Art", "ville": "New York, États-Unis",
        "site": "https://www.metmuseum.org", "couleur": "#c8102e",
        "licence": "Images et données CC0 (The Met Open Access)",
    },
    "washington": {
        "nom": "National Gallery of Art", "ville": "Washington, États-Unis",
        "site": "https://www.nga.gov", "couleur": "#2b4a7a",
        "licence": "Images et données CC0 (National Gallery of Art Open Access)",
    },
}

_impression = threading.Lock()


def dire(*morceaux):
    with _impression:
        print(*morceaux, flush=True)


# ─── Le réseau ────────────────────────────────────────────────────────────────

def lire(url, entetes=None, essais=4, delai=60):
    for essai in range(essais):
        try:
            requete = urllib.request.Request(url, headers={"User-Agent": AGENT, **(entetes or {})})
            with urllib.request.urlopen(requete, timeout=delai) as reponse:
                return reponse.read()
        except urllib.error.HTTPError as erreur:
            if erreur.code in (400, 403, 404, 410) or essai + 1 == essais:
                raise
            time.sleep(3 * (essai + 1) * (4 if erreur.code == 429 else 1))
        except Exception:
            if essai + 1 == essais:
                raise
            time.sleep(3 * (essai + 1))
    return b""


def json_de(url, entetes=None):
    return json.loads(lire(url, entetes))


# ─── Le texte ─────────────────────────────────────────────────────────────────

def propre(texte, limite=None):
    """Sans balises, sans espaces en trop ; coupé à la fin d'une phrase si trop long."""
    if not texte:
        return ""
    texte = html.unescape(re.sub(r"<[^>]+>", " ", str(texte)))
    texte = re.sub(r"\s+", " ", texte).strip()
    if limite and len(texte) > limite:
        coupe = texte[:limite]
        fin = max(coupe.rfind(". "), coupe.rfind("! "), coupe.rfind("? "))
        texte = coupe[:fin + 1] if fin > limite // 2 else coupe.rstrip() + "…"
    return texte


CATEGORIES = [
    ("Peinture", r"painting|paintings|tempera|oil on|fresco|icon"),
    ("Dessin", r"drawing|watercolor|watercolour|pastel|sketch|graphite|chalk"),
    ("Estampe", r"\bprint|etching|engraving|woodcut|lithograph|woodblock|aquatint|mezzotint|ukiyo"),
    ("Photographie", r"photograph|daguerreotype|albumen|gelatin silver|ambrotype|tintype|cyanotype"),
    ("Sculpture", r"sculpture|statue|statuette|relief|bust|figure|figurine"),
    ("Céramique", r"ceramic|porcelain|pottery|earthenware|stoneware|faience|vase|terracotta|ceramics"),
    ("Textile et costume", r"textile|costume|dress|fabric|embroider|tapestr|lace|kimono|garment|carpet|rug"),
    ("Mobilier", r"furniture|woodwork|chair|cabinet|table|desk"),
    ("Verre", r"glass"),
    ("Bijoux", r"jewel|ring|necklace|brooch|earring|pendant"),
    ("Armes et armures", r"arms|armor|armour|sword|helmet|weapon|dagger"),
    ("Livres et manuscrits", r"book|manuscript|folio|volume|codex|illuminat|calligraph"),
    ("Monnaies et médailles", r"coin|medal|plaquette"),
    ("Instruments de musique", r"musical|instrument"),
    ("Orfèvrerie et métal", r"metalwork|silver|gold|bronze|metal|enamel"),
]

POIDS = {
    "Peinture": 100, "Sculpture": 60, "Dessin": 45, "Livres et manuscrits": 40,
    "Photographie": 35, "Céramique": 35, "Textile et costume": 32, "Orfèvrerie et métal": 32,
    "Armes et armures": 32, "Bijoux": 30, "Mobilier": 30, "Verre": 28, "Estampe": 26,
    "Instruments de musique": 30, "Monnaies et médailles": 8, "Autres objets": 20,
}


def categorie(*textes):
    texte = " ".join(t for t in textes if t).lower()
    for nom, motif in CATEGORIES:
        if re.search(motif, texte):
            return nom
    return "Autres objets"


def annee(*valeurs):
    for v in valeurs:
        if isinstance(v, (int, float)) and v:
            return int(v)
        if isinstance(v, str):
            m = re.search(r"-?\d{3,4}", v)
            if m:
                return int(m.group())
    return None


def fiche(musee, ident, titre, artiste, **champs):
    """Une fiche, aux clés courtes (le fichier compte des dizaines de milliers de fiches)."""
    f = {"i": f"{musee}-{ident}", "t": propre(titre, 200) or "Sans titre", "a": propre(artiste, 120)}
    noms = {"vie": "v", "date": "d", "annee": "y", "technique": "m", "mesures": "s", "categorie": "c",
            "origine": "o", "texte": "x", "image": "u", "lien": "w", "ratio": "r", "salle": "g",
            "vignette": "_vignette", "score": "_score"}
    for cle, valeur in champs.items():
        if valeur not in (None, "", []):
            f[noms[cle]] = valeur
    return f


# ─── Les musées ───────────────────────────────────────────────────────────────

def chicago(combien, alea):
    base = "https://api.artic.edu/api/v1/artworks/search"
    entetes = {"AIC-User-Agent": AGENT}
    champs = ("id,title,artist_title,artist_display,date_display,date_start,medium_display,dimensions,"
              "image_id,classification_title,artwork_type_title,department_title,place_of_origin,"
              "description,short_description,is_boosted,gallery_title,thumbnail")
    commun = [("query[bool][must][0][term][is_public_domain]", "true"),
              ("query[bool][must][1][exists][field]", "image_id")]
    requetes = [
        commun + [("query[bool][must][2][term][is_boosted]", "true")],
        commun + [("query[bool][must][2][match][artwork_type_title]", "Painting")],
        commun + [("query[bool][must][2][match][artwork_type_title]", "Sculpture")],
        commun,
    ]
    vus = {}
    for requete in requetes:
        total = json_de(base + "?" + urllib.parse.urlencode(requete + [("limit", "1"), ("fields", "id")]), entetes)
        total = min(total["pagination"]["total"], 10000)
        pages = list(range(1, total // 100 + 2))
        if requete is requetes[-1]:
            alea.shuffle(pages)
        for page in pages:
            if len(vus) >= combien * 1.25:
                break
            try:
                lot = json_de(base + "?" + urllib.parse.urlencode(
                    requete + [("limit", "100"), ("page", str(page)), ("fields", champs)]), entetes)["data"]
            except Exception as erreur:
                dire(f"  chicago : page {page} refusée ({erreur})")
                time.sleep(5)
                continue
            for o in lot:
                if o["id"] in vus or not o.get("image_id"):
                    continue
                vignette = o.get("thumbnail") or {}
                cat = categorie(o.get("artwork_type_title"), o.get("classification_title"))
                vus[o["id"]] = fiche(
                    "chicago", o["id"], o.get("title"),
                    o.get("artist_title") or o.get("place_of_origin") or "Artiste inconnu",
                    vie=propre((o.get("artist_display") or "").split("\n")[-1], 160)
                    if "\n" in (o.get("artist_display") or "") else "",
                    date=propre(o.get("date_display"), 80), annee=annee(o.get("date_start"), o.get("date_display")),
                    technique=propre(o.get("medium_display"), 200), mesures=propre(o.get("dimensions"), 160),
                    categorie=cat, origine=propre(o.get("place_of_origin"), 80),
                    texte=propre(o.get("description") or o.get("short_description"), 900),
                    image=f"https://www.artic.edu/iiif/2/{o['image_id']}/full/843,/0/default.jpg",
                    vignette=f"https://www.artic.edu/iiif/2/{o['image_id']}/full/400,/0/default.jpg",
                    lien=f"https://www.artic.edu/artworks/{o['id']}",
                    ratio=round(vignette["height"] / vignette["width"], 3) if vignette.get("width") else None,
                    salle=propre(o.get("gallery_title"), 80),
                    score=POIDS[cat] + (200 if o.get("is_boosted") else 0),
                )
            time.sleep(1.1)  # l'API de Chicago : une requête par seconde
        dire(f"  chicago : {len(vus)} fiches")
    return list(vus.values())


def cleveland(combien, alea):
    base = "https://openaccess-api.clevelandart.org/api/artworks/"
    champs = ("id,title,creation_date,creation_date_earliest,creators,culture,technique,department,type,"
              "dimensions,images,url,wall_description,description,is_highlight,current_location")
    total = json_de(base + "?" + urllib.parse.urlencode({"cc0": "1", "has_image": "1", "limit": "1"}))["info"]["total"]
    fiches = []
    for depart in range(0, total, 1000):
        try:
            lot = json_de(base + "?" + urllib.parse.urlencode(
                {"cc0": "1", "has_image": "1", "limit": "1000", "skip": str(depart), "fields": champs}))["data"]
        except Exception as erreur:
            dire(f"  cleveland : lot {depart} refusé ({erreur})")
            continue
        for o in lot:
            web = (o.get("images") or {}).get("web") or {}
            if not web.get("url"):
                continue
            createurs = o.get("creators") or []
            if createurs:
                description = createurs[0].get("description") or ""
                artiste, _, vie = description.partition(" (")
                vie = vie.rstrip(")")
            else:
                artiste, vie = ", ".join(o.get("culture") or []) or "Artiste inconnu", ""
            cat = categorie(o.get("type"), o.get("department"), o.get("technique"))
            dimensions = o.get("dimensions")
            if isinstance(dimensions, dict):
                dimensions = ""
            largeur, hauteur = int(web.get("width") or 0), int(web.get("height") or 0)
            fiches.append(fiche(
                "cleveland", o["id"], o.get("title"), artiste, vie=propre(vie, 160),
                date=propre(o.get("creation_date"), 80),
                annee=annee(o.get("creation_date_earliest"), o.get("creation_date")),
                technique=propre(o.get("technique"), 200), mesures=propre(dimensions, 160), categorie=cat,
                origine=propre(", ".join(o.get("culture") or []), 80),
                texte=propre(o.get("wall_description") or o.get("description"), 900),
                image=web["url"], vignette=web["url"], lien=o.get("url"),
                ratio=round(hauteur / largeur, 3) if largeur else None,
                salle=propre(o.get("current_location"), 80),
                score=POIDS[cat] + (200 if o.get("is_highlight") else 0),
            ))
        time.sleep(0.5)
    dire(f"  cleveland : {len(fiches)} fiches")
    return fiches


def met(combien, alea):
    recherche = "https://collectionapi.metmuseum.org/public/collection/v1.1/search"
    identifiants, vus = [], set()

    def pages(filtres, maximum):
        trouves = []
        for depart in range(0, min(maximum, 10000), 100):
            try:
                lot = json_de(recherche + "?" + urllib.parse.urlencode(
                    {"q": "*", "hasImages": "true", **filtres, "limit": "100", "offset": str(depart)}))
            except Exception:
                break
            ids = lot.get("objectIDs") or []
            trouves += ids
            if len(ids) < 100:
                break
            time.sleep(0.1)
        return trouves

    # Les chefs-d'œuvre signalés, les peintures, puis tous les départements.
    groupes = [pages({"isHighlight": "true"}, 10000), pages({"departmentId": "11"}, 10000)]
    departements = [1, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 17, 18, 19]
    for d in departements:
        ids = pages({"departmentId": str(d)}, 3000)
        alea.shuffle(ids)
        groupes.append(ids)
    for groupe in groupes[:2]:
        for i in groupe:
            if i not in vus:
                vus.add(i)
                identifiants.append(i)
    # Les départements, à tour de rôle.
    reste = groupes[2:]
    while any(reste):
        for groupe in reste:
            if groupe:
                i = groupe.pop()
                if i not in vus:
                    vus.add(i)
                    identifiants.append(i)
    dire(f"  met : {len(identifiants)} identifiants")

    def objet(i):
        try:
            o = json_de(f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{i}")
        except Exception:
            return None
        if not (o.get("isPublicDomain") and o.get("primaryImageSmall")):
            return None
        cat = categorie(o.get("classification"), o.get("objectName"), o.get("department"))
        return fiche(
            "met", i, o.get("title"),
            o.get("artistDisplayName") or o.get("culture") or o.get("period") or "Artiste inconnu",
            vie=propre(o.get("artistDisplayBio"), 160), date=propre(o.get("objectDate"), 80),
            annee=annee(o.get("objectBeginDate"), o.get("objectDate")),
            technique=propre(o.get("medium"), 200), mesures=propre(o.get("dimensions"), 160), categorie=cat,
            origine=propre(o.get("culture") or o.get("country"), 80),
            image=o["primaryImageSmall"], vignette=o["primaryImageSmall"], lien=o.get("objectURL"),
            salle=f"salle {o['GalleryNumber']}" if o.get("GalleryNumber") else "",
            score=POIDS[cat] + (200 if o.get("isHighlight") else 0),
        )

    fiches = []
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        for depart in range(0, len(identifiants), 400):
            if len(fiches) >= combien * 1.2:
                break
            fiches += [f for f in pool.map(objet, identifiants[depart:depart + 400]) if f]
    dire(f"  met : {len(fiches)} fiches")
    return fiches


def washington(combien, alea, cache):
    depot = "https://raw.githubusercontent.com/NationalGalleryOfArt/opendata/main/data/"

    def table(nom):
        fichier = cache / f"nga-{nom}"
        if not fichier.exists() or time.time() - fichier.stat().st_mtime > 7 * 86400:
            fichier.write_bytes(lire(depot + nom, delai=300))
        return csv.DictReader(io.StringIO(fichier.read_text(encoding="utf-8")))

    images = {}
    for ligne in table("published_images.csv"):
        if ligne.get("openaccess") == "1" and ligne.get("viewtype") == "primary" and ligne.get("iiifurl"):
            images[ligne["depictstmsobjectid"]] = ligne
    fiches = []
    for o in table("objects.csv"):
        image = images.get(o["objectid"])
        if not image:
            continue
        cat = categorie(o.get("classification"), o.get("subclassification"), o.get("medium"))
        largeur, hauteur = int(image.get("width") or 0), int(image.get("height") or 0)
        fiches.append(fiche(
            "washington", o["objectid"], o.get("title"), o.get("attribution") or "Artiste inconnu",
            date=propre(o.get("displaydate"), 80), annee=annee(o.get("beginyear"), o.get("displaydate")),
            technique=propre(o.get("medium"), 200), mesures=propre(o.get("dimensions"), 160), categorie=cat,
            image=image["iiifurl"] + "/full/!1600,1600/0/default.jpg",
            vignette=image["iiifurl"] + f"/full/!{VIGNETTE},{VIGNETTE}/0/default.jpg",
            lien=f"https://www.nga.gov/collection/art-object-page.{o['objectid']}.html",
            ratio=round(hauteur / largeur, 3) if largeur else None,
            score=POIDS[cat],
        ))
    dire(f"  washington : {len(fiches)} fiches")
    return fiches


SOURCES = {"chicago": chicago, "cleveland": cleveland, "met": met, "washington": washington}


# ─── Le choix, les vignettes, les fichiers ────────────────────────────────────

def choisir(fiches, combien, alea):
    """Les meilleures fiches : le score du musée, un peu de hasard, et pas plus de 60
    œuvres par artiste (la collection reste variée)."""
    for f in fiches:
        f["_tri"] = f.get("_score", 0) + alea.random() * 25
    fiches.sort(key=lambda f: -f["_tri"])
    par_artiste, choix = {}, []
    for f in fiches:
        a = f.get("a") or "?"
        if par_artiste.get(a, 0) >= 60 and a != "Artiste inconnu":
            continue
        par_artiste[a] = par_artiste.get(a, 0) + 1
        choix.append(f)
        if len(choix) >= combien:
            break
    return choix


def vignette(f, dossier):
    """La vignette d'une fiche, sur le disque (téléchargée et réduite si besoin)."""
    fichier = dossier / f"{f['i']}.jpg"
    if fichier.exists() and fichier.stat().st_size > 0:
        return fichier.read_bytes()
    entetes = {"AIC-User-Agent": AGENT} if f["i"].startswith("chicago-") else None
    try:
        donnees = lire(f["_vignette"], entetes, essais=3, delai=40)
        image = Image.open(io.BytesIO(donnees))
        image.draft("RGB", (VIGNETTE * 2, VIGNETTE * 2))
        image = image.convert("RGB")
        image.thumbnail((VIGNETTE, VIGNETTE), Image.LANCZOS)
        sortie = io.BytesIO()
        image.save(sortie, "JPEG", quality=QUALITE, optimize=True)
        octets = sortie.getvalue()
        fichier.write_bytes(octets)
        if not f.get("r") and image.width:
            f["r"] = round(image.height / image.width, 3)
        return octets
    except Exception:
        return None


def moissonner(cle, combien, cache, alea):
    fichier = cache / f"fiches-{cle}.json"
    if fichier.exists() and time.time() - fichier.stat().st_mtime < 7 * 86400:
        fiches = json.loads(fichier.read_text(encoding="utf-8"))
        dire(f"• {MUSEES[cle]['nom']} : {len(fiches)} fiches du cache")
    else:
        dire(f"• {MUSEES[cle]['nom']}")
        source = SOURCES[cle]
        fiches = source(combien, alea, cache) if cle == "washington" else source(combien, alea)
        if fiches:
            fichier.write_text(json.dumps(fiches, ensure_ascii=False), encoding="utf-8")
    return fiches


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--sortie", default="ios/ARTEON.swiftpm/Musee/collections")
    parser.add_argument("--cache", default="target/collections-cache")
    parser.add_argument("--total", type=int, default=50000, help="le nombre d'œuvres voulu")
    parser.add_argument("--musees", default=",".join(PARTS), help="chicago,cleveland,met,washington")
    parser.add_argument("--fils", type=int, default=24, help="téléchargements en parallèle")
    args = parser.parse_args()

    sortie, cache = Path(args.sortie), Path(args.cache)
    sortie.mkdir(parents=True, exist_ok=True)
    (cache / "vignettes").mkdir(parents=True, exist_ok=True)
    cles = [c for c in args.musees.split(",") if c in SOURCES]
    debut = time.time()

    # Les fiches des musées, en parallèle.
    parts = {c: max(1, round(args.total * PARTS[c] / sum(PARTS[k] for k in cles))) for c in cles}
    with concurrent.futures.ThreadPoolExecutor(len(cles)) as pool:
        futures = {c: pool.submit(moissonner, c, parts[c], cache, random.Random(f"arteon-{c}")) for c in cles}
        toutes = {}
        for c, future in futures.items():
            try:
                toutes[c] = future.result()
            except Exception as erreur:
                dire(f"  {MUSEES[c]['nom']} est sauté : {erreur}")
                toutes[c] = []

    # Un musée qui manque laisse sa part aux autres.
    manque = sum(parts[c] for c in cles if not toutes[c])
    vivants = [c for c in cles if toutes[c]]
    for c in vivants:
        parts[c] += manque // max(1, len(vivants))

    def traiter(c):
        """Les vignettes d'un musée (téléchargées, réduites) et son paquet."""
        choix = choisir(toutes[c], round(parts[c] * 1.08), random.Random(f"choix-{c}"))
        dossier = cache / "vignettes"
        with concurrent.futures.ThreadPoolExecutor(args.fils) as pool:
            octets = list(pool.map(lambda f: vignette(f, dossier), choix))
        paquet = bytearray()
        gardees = []
        for f, o in zip(choix, octets):
            if not o:
                continue
            f["p"] = [len(paquet), len(o)]
            paquet += o
            gardees.append({k: v for k, v in f.items() if not k.startswith("_")})
            if len(gardees) >= parts[c]:
                break
        (sortie / f"vignettes-{c}.bin").write_bytes(bytes(paquet))
        for f in gardees:
            f["k"] = c
        dire(f"  {MUSEES[c]['nom']} : {len(gardees)} œuvres, vignettes {len(paquet) / 1e6:.0f} Mo")
        return {"cle": c, **MUSEES[c], "oeuvres": len(gardees), "paquet": f"vignettes-{c}.bin"}, gardees

    # Les quatre musées en même temps.
    musees, fiches_finales = [], []
    with concurrent.futures.ThreadPoolExecutor(max(1, len(vivants))) as pool:
        for musee, gardees in pool.map(traiter, vivants):
            musees.append(musee)
            fiches_finales += gardees

    catalogue = {
        "version": VERSION,
        "date": datetime.date.today().isoformat(),
        "musees": musees,
        "oeuvres": fiches_finales,
    }
    (sortie / "collections.json").write_text(
        json.dumps(catalogue, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    taille = sum(p.stat().st_size for p in sortie.iterdir())
    dire(f"✓ collections : {len(fiches_finales)} œuvres de {len(musees)} musées, "
         f"{taille / 1e6:.0f} Mo, en {(time.time() - debut) / 60:.0f} min")
    if not fiches_finales:
        sys.exit(1)


if __name__ == "__main__":
    main()
