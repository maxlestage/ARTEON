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
# L'image en grand, embarquée elle aussi (l'app est 100 % hors ligne) : 800 px, ~45 Ko.
GRANDE = 800
QUALITE_GRANDE = 72
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

# L'heure à laquelle on arrête de demander des fiches (--minutes) : la moisson garde ce
# qu'elle a, et la suivante continue grâce au cache.
FIN_DES_FICHES = float("inf")


def trop_tard():
    return time.time() > FIN_DES_FICHES


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
            # 403 et 429 : le musée ralentit qui va trop vite (les machines de GitHub
            # surtout) ; on attend, de plus en plus longtemps, et on recommence.
            if erreur.code in (400, 404, 410) or essai + 1 == essais:
                raise
            time.sleep(min(90, 8 * 2 ** essai) if erreur.code in (403, 429) else 3 * (essai + 1))
        except Exception:
            if essai + 1 == essais:
                raise
            time.sleep(3 * (essai + 1))
    return b""


def json_de(url, entetes=None, essais=6):
    return json.loads(lire(url, entetes, essais=essais))


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

    def ajouter(lot):
        _chicago_fiches(lot, vus)

    # La recherche de Chicago ne rend jamais plus de 1 000 résultats par requête : on la
    # découpe en tranches d'identifiants (jusqu'à 290 000), assez étroites pour en rester
    # sous ce nombre.
    for n, requete in enumerate(requetes):
        largeur = 25000 if n < len(requetes) - 1 else 2500
        tranches = [(debut, debut + largeur) for debut in range(0, 290000, largeur)]
        if n == len(requetes) - 1:
            alea.shuffle(tranches)
        for debut, fin in tranches:
            if len(vus) >= combien * 1.25 or trop_tard():
                break
            borne = len(requete)
            tranche = requete + [(f"query[bool][must][{borne}][range][id][gte]", str(debut)),
                                 (f"query[bool][must][{borne}][range][id][lt]", str(fin))]
            for page in range(1, 11):
                try:
                    lot = json_de(base + "?" + urllib.parse.urlencode(
                        tranche + [("limit", "100"), ("page", str(page)), ("fields", champs)]), entetes)["data"]
                except Exception as erreur:
                    dire(f"  chicago : tranche {debut}, page {page} refusée ({erreur})")
                    break
                ajouter(lot)
                time.sleep(1.2)  # l'API de Chicago : pas plus d'une requête par seconde
                if len(lot) < 100:
                    break
        dire(f"  chicago : {len(vus)} fiches")
    return list(vus.values())


def _chicago_fiches(lot, vus):
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


def met(combien, alea, deja=frozenset()):
    recherche = "https://collectionapi.metmuseum.org/public/collection/v1.1/search"
    identifiants, vus = [], set()

    def pages(filtres, maximum):
        trouves = []
        for depart in range(0, min(maximum, 10000), 100):
            if trop_tard():
                break
            try:
                lot = json_de(recherche + "?" + urllib.parse.urlencode(
                    {"q": "*", "hasImages": "true", **filtres, "limit": "100", "offset": str(depart)}))
            except Exception:
                break
            ids = lot.get("objectIDs") or []
            trouves += ids
            if len(ids) < 100:
                break
            time.sleep(0.4)
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
        if trop_tard():
            return None
        time.sleep(0.12)  # le Met : au plus quelques dizaines de requêtes par seconde
        try:
            o = json_de(f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{i}", essais=3)
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

    # Ceux déjà dans le cache ne sont pas redemandés : chaque moisson en ajoute d'autres.
    identifiants = [i for i in identifiants if f"met-{i}" not in deja]
    fiches = []
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        for depart in range(0, len(identifiants), 400):
            if len(fiches) >= combien * 1.2 or trop_tard():
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


def source(f):
    """L'image à télécharger : assez grande pour en tirer l'image de 800 px."""
    if f["i"].startswith("chicago-") and f.get("u"):
        return f["u"]  # 843 px, la largeur que l'IIIF de Chicago sert
    if f["i"].startswith("washington-") and f.get("u"):
        return f["u"].replace("/full/!1600,1600/", f"/full/!{GRANDE},{GRANDE}/")
    return f["_vignette"]


def reduire(image, cote, qualite):
    copie = image.copy()
    copie.thumbnail((cote, cote), Image.LANCZOS)
    sortie = io.BytesIO()
    copie.save(sortie, "JPEG", quality=qualite, optimize=True, progressive=cote > VIGNETTE)
    return sortie.getvalue(), copie


def vignette(f, dossier):
    """La vignette d'une fiche (320 px) et son image en grand (800 px), sur le disque :
    téléchargées une fois, réduites, gardées dans le cache. Renvoie la vignette."""
    petite = dossier / f"{f['i']}.jpg"
    grande = dossier.parent / "grandes" / f"{f['i']}.jpg"
    if petite.exists() and petite.stat().st_size > 0 and grande.exists() and grande.stat().st_size > 0:
        return petite.read_bytes()
    entetes = {"AIC-User-Agent": AGENT} if f["i"].startswith("chicago-") else None
    try:
        donnees = lire(source(f), entetes, essais=3, delai=60)
        image = Image.open(io.BytesIO(donnees))
        image.draft("RGB", (GRANDE * 2, GRANDE * 2))
        image = image.convert("RGB")
        octets_grande, _ = reduire(image, GRANDE, QUALITE_GRANDE)
        octets, reduite = reduire(image, VIGNETTE, QUALITE)
        grande.parent.mkdir(parents=True, exist_ok=True)
        grande.write_bytes(octets_grande)
        petite.write_bytes(octets)
        if not f.get("r") and reduite.width:
            f["r"] = round(reduite.height / reduite.width, 3)
        return octets
    except Exception:
        return None


def moissonner(cle, combien, cache, alea):
    fichier = cache / f"fiches-{cle}.json"
    fiches = []
    if fichier.exists() and time.time() - fichier.stat().st_mtime < 7 * 86400:
        fiches = json.loads(fichier.read_text(encoding="utf-8"))
    # Un cache trop maigre (une moisson gênée par le musée) est refait.
    if len(fiches) >= combien:
        dire(f"• {MUSEES[cle]['nom']} : {len(fiches)} fiches du cache")
    else:
        dire(f"• {MUSEES[cle]['nom']}")
        source = SOURCES[cle]
        if cle == "washington":
            nouvelles = source(combien, alea, cache)
        elif cle == "met":
            nouvelles = source(max(1, combien - len(fiches)), alea, frozenset(f["i"] for f in fiches))
        else:
            nouvelles = source(combien, alea)
        # Les fiches d'avant et les nouvelles, réunies (une moisson interrompue continue).
        connues = {f["i"]: f for f in fiches}
        connues.update({f["i"]: f for f in nouvelles})
        fiches = list(connues.values())
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
    parser.add_argument("--minutes", type=float, default=0,
                        help="au plus tant de minutes pour les fiches (0 : sans limite)")
    args = parser.parse_args()
    global FIN_DES_FICHES
    if args.minutes:
        FIN_DES_FICHES = time.time() + args.minutes * 60

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

    # Chaque musée télécharge ses vignettes dans l'ordre de ses candidates, jusqu'à sa
    # part : une vignette refusée est remplacée par la suivante. Ce qui manque encore à la
    # fin (un musée qui refuse beaucoup, un autre trop lent) est pris chez ceux qui ont
    # encore des candidates : le total voulu est atteint.
    vivants = [c for c in cles if toutes[c]]
    classees = {c: choisir(toutes[c], len(toutes[c]), random.Random(f"choix-{c}")) for c in vivants}
    position = {c: 0 for c in vivants}
    gardees = {c: [] for c in vivants}
    octets = {c: [] for c in vivants}
    dossier = cache / "vignettes"

    def remplir(c, cible):
        with concurrent.futures.ThreadPoolExecutor(args.fils) as pool:
            while len(gardees[c]) < cible and position[c] < len(classees[c]):
                manque = cible - len(gardees[c])
                lot = classees[c][position[c]:position[c] + max(manque + manque // 8, 24)]
                position[c] += len(lot)
                for f, o in zip(lot, pool.map(lambda f: vignette(f, dossier), lot)):
                    if o and len(gardees[c]) < cible:
                        gardees[c].append(f)
                        octets[c].append(o)
        return len(gardees[c])

    parts = {c: parts[c] + sum(parts[k] for k in cles if k not in vivants) // max(1, len(vivants))
             for c in vivants}
    with concurrent.futures.ThreadPoolExecutor(max(1, len(vivants))) as pool:
        list(pool.map(lambda c: remplir(c, parts[c]), vivants))
    for _ in range(4):
        manque = args.total - sum(len(gardees[c]) for c in vivants)
        receveurs = [c for c in vivants if position[c] < len(classees[c])]
        if manque <= 0 or not receveurs:
            break
        dire(f"  il manque {manque} œuvres : {', '.join(receveurs)} complètent")
        part = -(-manque // len(receveurs))
        with concurrent.futures.ThreadPoolExecutor(len(receveurs)) as pool:
            list(pool.map(lambda c: remplir(c, len(gardees[c]) + part), receveurs))

    musees, fiches_finales = [], []
    surplus = sum(len(gardees[c]) for c in vivants) - args.total
    for c in vivants:
        # Le dernier lot peut dépasser le total de quelques œuvres : on les retire au
        # musée le plus fourni.
        if surplus > 0 and len(gardees[c]) == max(len(gardees[k]) for k in vivants):
            del gardees[c][-surplus:], octets[c][-surplus:]
            surplus = 0
        paquet = bytearray()
        for f, o in zip(gardees[c], octets[c]):
            f["p"] = [len(paquet), len(o)]
            f["k"] = c
            paquet += o
        (sortie / f"vignettes-{c}.bin").write_bytes(bytes(paquet))
        # Les images en grand, bout à bout elles aussi, écrites au fil de l'eau (2 Go au total).
        taille_grandes = 0
        with open(sortie / f"grandes-{c}.bin", "wb") as grandes:
            for f in gardees[c]:
                chemin = cache / "grandes" / f"{f['i']}.jpg"
                if chemin.exists():
                    octets_grande = chemin.read_bytes()
                    f["gr"] = [taille_grandes, len(octets_grande)]
                    grandes.write(octets_grande)
                    taille_grandes += len(octets_grande)
        fiches = [{k: v for k, v in f.items() if not k.startswith("_")} for f in gardees[c]]
        fiches_finales += fiches
        musees.append({"cle": c, **MUSEES[c], "oeuvres": len(fiches), "paquet": f"vignettes-{c}.bin",
                       "grandes": f"grandes-{c}.bin"})
        dire(f"  {MUSEES[c]['nom']} : {len(fiches)} œuvres, vignettes {len(paquet) / 1e6:.0f} Mo, "
             f"images en grand {taille_grandes / 1e6:.0f} Mo")

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
