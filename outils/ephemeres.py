#!/usr/bin/env python3
"""Les expositions éphémères d'ARTEON.

Chaque semaine, ce script interroge les collections ouvertes de grands musées du monde et
monte, pour chacun, une exposition de douze œuvres tirées au sort parmi celles qu'il
expose EN CE MOMENT dans ses salles — uniquement des œuvres du domaine public, dont le
musée publie la photographie en CC0. Quand le musée change son accrochage (une œuvre
part en prêt, une autre sort des réserves), l'exposition change avec lui.

    python3 outils/ephemeres.py              # le tirage de la semaine en cours
    python3 outils/ephemeres.py --graine 42  # un autre tirage

Il écrit `expositions/ephemere-<ville>.toml` et les images dans
`public/tableaux/ephemere-<ville>/` (fichiers ignorés par git : ils sont refaits à
chaque publication du site). Il n'utilise que la bibliothèque standard de Python. Un
musée qui ne répond pas est simplement sauté ; le site se construit sans lui.
"""

import argparse
import datetime
import html
import json
import random
import re
import shutil
import struct
import subprocess
import sys
import time
import unicodedata
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
AGENT = "ARTEON/1.0 (https://github.com/maxlestage/ARTEON)"
PAR_EXPOSITION = 12


# ─── Le réseau ────────────────────────────────────────────────────────────────

def lire(url, entetes=None, essais=3):
    """Le corps d'une réponse HTTP, avec quelques nouvelles tentatives."""
    for essai in range(essais):
        try:
            requete = urllib.request.Request(url, headers={"User-Agent": AGENT, **(entetes or {})})
            with urllib.request.urlopen(requete, timeout=40) as reponse:
                return reponse.read()
        except urllib.error.HTTPError as erreur:
            # Certains serveurs (l'IIIF de Chicago) refusent Python mais acceptent curl.
            if erreur.code == 403 and shutil.which("curl"):
                commande = ["curl", "-sSfL", "-m", "60", "-A", AGENT]
                for nom, valeur in (entetes or {}).items():
                    commande += ["-H", f"{nom}: {valeur}"]
                resultat = subprocess.run(commande + [url], capture_output=True)
                if resultat.returncode == 0:
                    return resultat.stdout
            if essai + 1 == essais:
                raise
            print(f"  … nouvel essai ({erreur})", file=sys.stderr)
            time.sleep(4 * (essai + 1))
        except Exception as erreur:  # réseau, 429, 5xx…
            if essai + 1 == essais:
                raise
            print(f"  … nouvel essai ({erreur})", file=sys.stderr)
            time.sleep(4 * (essai + 1))
    return b""


def json_de(url, entetes=None):
    return json.loads(lire(url, entetes))


def taille_jpeg(donnees):
    """(largeur, hauteur) d'une image JPEG, lues dans son en-tête."""
    if donnees[:2] != b"\xff\xd8":
        return None
    i = 2
    while i + 9 < len(donnees):
        if donnees[i] != 0xFF:
            i += 1
            continue
        marqueur = donnees[i + 1]
        if marqueur in (0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF):
            hauteur, largeur = struct.unpack(">HH", donnees[i + 5:i + 9])
            return largeur, hauteur
        longueur = struct.unpack(">H", donnees[i + 2:i + 4])[0]
        i += 2 + longueur
    return None


# ─── Le texte en français ─────────────────────────────────────────────────────

ROMAINS = ["", "Iᵉʳ", "IIᵉ", "IIIᵉ", "IVᵉ", "Vᵉ", "VIᵉ", "VIIᵉ", "VIIIᵉ", "IXᵉ", "Xᵉ", "XIᵉ",
           "XIIᵉ", "XIIIᵉ", "XIVᵉ", "XVᵉ", "XVIᵉ", "XVIIᵉ", "XVIIIᵉ", "XIXᵉ", "XXᵉ", "XXIᵉ"]


def date_fr(date):
    """« c. 1890 », « 19th century », « 900–400 B.C. » en français."""
    d = (date or "").strip()
    d = re.sub(r"\b(c\.|ca\.|circa|about)\s*", "vers ", d, flags=re.I)
    d = re.sub(r"\bearly\s+", "début du ", d, flags=re.I)
    d = re.sub(r"\blate\s+", "fin du ", d, flags=re.I)
    d = re.sub(r"\bmid-?\s*", "milieu du ", d, flags=re.I)

    def siecle(m):
        n = int(m.group(1))
        return (ROMAINS[n] if n < len(ROMAINS) else f"{n}ᵉ") + " siècle"

    d = re.sub(r"(\d{1,2})(?:st|nd|rd|th)\s+century", siecle, d, flags=re.I)
    d = re.sub(r"\b(B\.?C\.?E?\.?|BCE)\b", "av. J.-C.", d)
    d = re.sub(r"\b(A\.?D\.?|CE)\b", "apr. J.-C.", d)
    d = re.sub(r"(\d)-(\d)", r"\1–\2", d).replace(" or ", " ou ")
    return d[:1].upper() + d[1:] if d else "Date inconnue"


TECHNIQUES = {
    "oil on canvas": "Huile sur toile", "oil on panel": "Huile sur bois", "oil on wood": "Huile sur bois",
    "oil on wood panel": "Huile sur bois", "oil on oak panel": "Huile sur chêne", "oil on paper": "Huile sur papier",
    "oil on board": "Huile sur carton", "oil on cardboard": "Huile sur carton", "oil on copper": "Huile sur cuivre",
    "tempera on panel": "Tempera sur bois", "tempera and gold on panel": "Tempera et or sur bois",
    "tempera on wood, gold ground": "Tempera sur bois, fond d'or", "watercolor": "Aquarelle",
    "watercolor on paper": "Aquarelle sur papier", "pastel": "Pastel", "pastel on paper": "Pastel sur papier",
    "bronze": "Bronze", "marble": "Marbre", "terracotta": "Terre cuite", "earthenware": "Terre cuite",
    "stoneware": "Grès", "porcelain": "Porcelaine", "hard-paste porcelain": "Porcelaine dure",
    "glass": "Verre", "gold": "Or", "silver": "Argent", "ivory": "Ivoire", "jade": "Jade", "wood": "Bois",
    "limestone": "Calcaire", "sandstone": "Grès", "granite": "Granit", "alabaster": "Albâtre",
    "silk": "Soie", "wool": "Laine", "cotton": "Coton", "linen": "Lin",
    "gelatin silver print": "Épreuve gélatino-argentique", "albumen print": "Épreuve sur papier albuminé",
    "albumen silver print": "Épreuve sur papier albuminé", "salted paper print": "Épreuve sur papier salé",
    "daguerreotype": "Daguerréotype", "etching": "Eau-forte", "engraving": "Gravure au burin",
    "lithograph": "Lithographie", "color lithograph": "Lithographie en couleurs",
    "woodblock print": "Gravure sur bois", "color woodblock print": "Gravure sur bois en couleurs",
    "woodblock print; ink and color on paper": "Gravure sur bois, encre et couleurs sur papier",
    "ink on paper": "Encre sur papier", "ink and color on paper": "Encre et couleurs sur papier",
    "ink and color on silk": "Encre et couleurs sur soie", "ink on silk": "Encre sur soie",
    "graphite": "Mine de plomb", "charcoal": "Fusain", "red chalk": "Sanguine",
}


def technique_fr(medium):
    m = re.sub(r"\s+", " ", (medium or "").strip())
    return TECHNIQUES.get(m.lower().rstrip("."), m[:1].upper() + m[1:] if m else "")


def dimensions_cm(texte):
    """La première mesure en cm, « 73,7 × 92,1 cm », et sa largeur en mètres (ou None)."""
    t = (texte or "").replace("×", "x")
    m = re.search(r"(\d+(?:[.,]\d+)?)\s*x\s*(\d+(?:[.,]\d+)?)(?:\s*x\s*(\d+(?:[.,]\d+)?))?\s*cm", t)
    if not m:
        return "", None
    hauteur, largeur = float(m.group(1).replace(",", ".")), float(m.group(2).replace(",", "."))
    lisible = f"{m.group(1)} × {m.group(2)} cm".replace(".", ",")
    return lisible, largeur / 100.0


def sans_balises(texte):
    t = re.sub(r"<[^>]+>", " ", texte or "")
    return re.sub(r"\s+", " ", html.unescape(t)).strip()


def nom_de_fichier(titre, pris):
    s = unicodedata.normalize("NFKD", titre).encode("ascii", "ignore").decode().lower()
    s = re.sub(r"[^a-z0-9]+", "-", s).strip("-")[:48].strip("-") or "oeuvre"
    base, n = s, 2
    while s in pris:
        s, n = f"{base}-{n}", n + 1
    pris.add(s)
    return s


def chaine(texte):
    """Une chaîne TOML (le JSON en est un sous-ensemble valable pour nos textes)."""
    return json.dumps(texte, ensure_ascii=False)


def cadre_pour(genre):
    g = (genre or "").lower()
    if "photograph" in g:
        return "noir"
    if any(k in g for k in ("print", "drawing", "manuscript", "book", "album", "watercolor")):
        return "estampe"
    if any(k in g for k in ("textile", "costume", "carpet", "tapestry")):
        return "aucun"
    if "painting" in g:
        return "dore"
    return "vitrine"


# ─── Les musées ───────────────────────────────────────────────────────────────

def chicago(alea, combien):
    base = "https://api.artic.edu/api/v1/artworks/search"
    champs = ("id,title,artist_title,artist_display,date_display,medium_display,dimensions,image_id,"
              "gallery_title,short_description,description,classification_title,thumbnail,place_of_origin")
    filtre = {"query[bool][must][0][term][is_on_view]": "true",
              "query[bool][must][1][term][is_public_domain]": "true",
              "query[bool][must][2][exists][field]": "image_id", "fields": champs, "limit": "50"}
    premiere = json_de(base + "?" + urllib.parse.urlencode({**filtre, "limit": "1"}))
    pages = max(1, min(premiere["pagination"]["total"] // 50, 200))
    candidats = []
    for page in alea.sample(range(1, pages + 1), min(4, pages)):
        try:
            candidats += json_de(base + "?" + urllib.parse.urlencode({**filtre, "page": str(page)}))["data"]
        except Exception as erreur:  # une page refusée : on garde les autres
            print(f"  page {page} refusée ({erreur})", file=sys.stderr)
        time.sleep(1.5)
    oeuvres = []
    for o in candidats:
        vignette = o.get("thumbnail") or {}
        if not o.get("image_id") or not vignette.get("width"):
            continue
        texte = sans_balises(o.get("description") or o.get("short_description") or "")
        oeuvres.append({
            "titre": o["title"], "artiste": o.get("artist_title") or (o.get("place_of_origin") or "Artiste inconnu"),
            "date": o.get("date_display"), "technique": o.get("medium_display"), "mesures": o.get("dimensions"),
            "salle": o.get("gallery_title") or "", "genre": o.get("classification_title") or "",
            "texte": texte, "lien": f"https://www.artic.edu/artworks/{o['id']}",
            "image": f"https://www.artic.edu/iiif/2/{o['image_id']}/full/1000,/0/default.jpg",
            "entetes": {"AIC-User-Agent": AGENT},
            "ratio": vignette["height"] / vignette["width"],
        })
    return oeuvres


def cleveland(alea, combien):
    base = "https://openaccess-api.clevelandart.org/api/artworks/"
    filtre = {"cc0": "1", "has_image": "1", "currently_on_view": "1", "limit": "1"}
    total = json_de(base + "?" + urllib.parse.urlencode(filtre))["info"]["total"]
    oeuvres = []
    for depart in alea.sample(range(0, max(1, total - 40)), 3):
        lot = json_de(base + "?" + urllib.parse.urlencode({**filtre, "limit": "40", "skip": str(depart)}))
        for o in lot["data"]:
            web = (o.get("images") or {}).get("web") or {}
            if not web.get("url") or not web.get("width"):
                continue
            createurs = o.get("creators") or []
            artiste = createurs[0]["description"].split(" (")[0] if createurs else ", ".join(o.get("culture") or []) or "Artiste inconnu"
            oeuvres.append({
                "titre": o["title"], "artiste": artiste, "date": o.get("creation_date"),
                "technique": o.get("technique"), "mesures": o.get("measurements"),
                "salle": o.get("current_location") or "", "genre": o.get("type") or "",
                "texte": sans_balises(o.get("wall_description") or o.get("description") or ""),
                "lien": o.get("url") or "", "image": web["url"], "entetes": {},
                "ratio": int(web["height"]) / int(web["width"]),
            })
        time.sleep(0.5)
    return oeuvres


def new_york(alea, combien):
    recherche = "https://collectionapi.metmuseum.org/public/collection/v1.1/search?q=*&isOnView=true&hasImages=true"
    identifiants = json_de(recherche).get("objectIDs") or []
    oeuvres = []
    for ident in alea.sample(identifiants, min(90, len(identifiants))):
        if len(oeuvres) >= combien * 3:
            break
        try:
            o = json_de(f"https://collectionapi.metmuseum.org/public/collection/v1/objects/{ident}")
        except Exception:
            continue
        time.sleep(0.15)
        if not (o.get("isPublicDomain") and o.get("GalleryNumber") and o.get("primaryImageSmall")):
            continue
        artiste = o.get("artistDisplayName") or o.get("culture") or o.get("period") or "Artiste inconnu"
        oeuvres.append({
            "titre": o["title"], "artiste": artiste, "date": o.get("objectDate"), "technique": o.get("medium"),
            "mesures": o.get("dimensions"), "salle": f"salle {o['GalleryNumber']}",
            "genre": o.get("classification") or o.get("objectName") or "", "texte": "",
            "lien": o.get("objectURL") or "", "image": o["primaryImageSmall"], "entetes": {}, "ratio": None,
        })
    return oeuvres


MUSEES = [
    {
        "slug": "ephemere-chicago", "titre": "En ce moment à l'Art Institute of Chicago", "ville": "Chicago", "ordre": 401, "couleur": "#b5121b", "source": chicago,
        "musee": "Art Institute of Chicago", "lieu": "Chicago, États-Unis",
        "architecte": "Shepley, Rutan and Coolidge ; Renzo Piano (Modern Wing)", "construction": "1892–1893 ; 2009",
        "presentation": (
            "Bâti pour l'Exposition universelle de 1893 sur Michigan Avenue, gardé par deux lions de bronze, "
            "l'Art Institute of Chicago est l'un des plus grands musées d'art des États-Unis. Ses salles "
            "réunissent un ensemble célèbre de peintures impressionnistes et postimpressionnistes — la Grande "
            "Jatte de Seurat y est depuis 1926 —, mais aussi de l'art d'Asie, d'Afrique, des Amériques et "
            "d'Europe, des armes, des arts décoratifs, de la photographie. En 2009, l'aile moderne de Renzo "
            "Piano, baignée de lumière, a agrandi le musée. Il publie librement en ligne les images de ses "
            "œuvres du domaine public, et dit lesquelles sont exposées dans ses salles."),
    },
    {
        "slug": "ephemere-cleveland", "titre": "En ce moment au musée d'art de Cleveland", "ville": "Cleveland", "ordre": 402, "couleur": "#1f5c4a", "source": cleveland,
        "musee": "Cleveland Museum of Art", "lieu": "Cleveland, États-Unis",
        "architecte": "Hubbell & Benes ; Rafael Viñoly (agrandissement)", "construction": "1916 ; 2013",
        "presentation": (
            "Ouvert en 1916 dans un temple néoclassique de marbre blanc, au bord du Wade Park, le Cleveland "
            "Museum of Art est gratuit depuis toujours. Ses collections couvrent cinq mille ans d'art, de "
            "l'Égypte ancienne à l'art contemporain, avec des ensembles remarquables d'art médiéval, d'art "
            "asiatique et de peinture européenne. En 2013, l'agrandissement de Rafael Viñoly a relié les "
            "bâtiments par un vaste atrium couvert. En 2019, le musée a placé sous licence CC0 les images de "
            "toutes ses œuvres du domaine public, et il indique celles qui sont exposées."),
    },
    {
        "slug": "ephemere-new-york", "titre": 'En ce moment au Metropolitan Museum de New York', "ville": "New York", "ordre": 403, "couleur": "#c8102e", "source": new_york,
        "musee": "Metropolitan Museum of Art", "lieu": "New York, États-Unis",
        "architecte": "Calvert Vaux et Jacob Wrey Mould ; Richard Morris Hunt (façade)", "construction": "1880 ; 1902",
        "presentation": (
            "Fondé en 1870, le Metropolitan Museum of Art s'installe en 1880 dans Central Park, dans un "
            "bâtiment de Calvert Vaux et Jacob Wrey Mould, puis prend en 1902 sa grande façade Beaux-Arts sur "
            "la Cinquième Avenue. Le « Met » est l'un des plus grands musées du monde : cinq mille ans d'art "
            "de tous les continents, du temple égyptien de Dendour aux armures, aux instruments de musique, "
            "aux costumes et aux peintures européennes. Depuis 2017, il offre en CC0 les images de ses œuvres "
            "du domaine public, et indique la salle où chacune est exposée."),
    },
]


# ─── L'exposition ─────────────────────────────────────────────────────────────

def choisir(oeuvres, alea, combien):
    """Douze œuvres variées : pas plus de deux par artiste, ni de quatre du même genre."""
    alea.shuffle(oeuvres)
    par_artiste, par_genre, choix = {}, {}, []
    for o in oeuvres:
        a, g = o["artiste"], (o["genre"] or "?").lower()
        if par_artiste.get(a, 0) >= 2 or par_genre.get(g, 0) >= 4:
            continue
        par_artiste[a] = par_artiste.get(a, 0) + 1
        par_genre[g] = par_genre.get(g, 0) + 1
        choix.append(o)
        if len(choix) == combien:
            break
    return choix


def largeur_pour(oeuvre, ratio):
    """La largeur affichée : la vraie pour un tableau (si elle est connue), agrandie pour un objet."""
    _, vraie = dimensions_cm(oeuvre["mesures"])
    if cadre_pour(oeuvre["genre"]) in ("dore", "aucun") and vraie:
        largeur = vraie
    else:
        largeur = (1.1 / max(ratio, 0.2)) ** 0.5
    largeur = max(0.7, min(largeur, 1.8))
    # La hauteur reste sous 2,6 m (les salles font 5 m).
    return round(min(largeur, 2.6 / max(ratio, 0.1)), 2)


def monter(musee, semaine, graine):
    alea = random.Random(f"{graine}-{musee['slug']}")
    print(f"• {musee['musee']}")
    oeuvres = choisir(musee["source"](alea, PAR_EXPOSITION), alea, PAR_EXPOSITION)
    dossier = RACINE / "public" / "tableaux" / musee["slug"]
    shutil.rmtree(dossier, ignore_errors=True)
    dossier.mkdir(parents=True)
    pris, accrochees = set(), []
    for o in oeuvres:
        try:
            image = lire(o["image"], o["entetes"])
        except Exception as erreur:
            print(f"  image introuvable : {o['titre']} ({erreur})", file=sys.stderr)
            continue
        taille = taille_jpeg(image)
        if not taille or taille[0] < 300:
            continue
        nom = nom_de_fichier(o["titre"], pris)
        (dossier / f"{nom}.jpg").write_bytes(image)
        accrochees.append((o, nom, taille[1] / taille[0]))
        time.sleep(0.3)
    if len(accrochees) < 6:
        shutil.rmtree(dossier, ignore_errors=True)
        raise RuntimeError(f"seulement {len(accrochees)} œuvres utilisables")

    lundi = datetime.date.fromisocalendar(semaine[0], semaine[1], 1)
    mois = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre",
            "octobre", "novembre", "décembre"][lundi.month - 1]
    jour = f"{lundi.day} {mois} {lundi.year}"
    annees = [int(a) for o, _, _ in accrochees for a in re.findall(r"\b(\d{4})\b", o["date"] or "")]
    dates = f"{min(annees)}–{max(annees)}" if annees else "Toutes époques"
    salles = (len(accrochees) + 3) // 4
    noms_salles = ["Première salle", "Deuxième salle", "Troisième salle", "Quatrième salle"]
    lignes = [
        "# Exposition éphémère, écrite par outils/ephemeres.py : ne pas modifier à la main.",
        "", "[exposition]", f"ordre = {musee['ordre']}", f"nom_court = {chaine('Éphémère · ' + musee['ville'])}",
        f"mouvement = {chaine('Exposition éphémère · ' + musee['ville'])}", "ephemere = true",
        f"cadre = {chaine('dore')}", f"couleur = {chaine(musee['couleur'])}",
        f"titre = {chaine(musee['titre'])}",
        f"artiste = {chaine('Artistes du monde entier')}", f"dates = {chaine(dates)}",
        f"accroche = {chaine('Exposition éphémère · ' + musee['musee'] + ' · semaine du ' + jour)}",
        "introduction = " + chaine(
            f"Une exposition qui change chaque semaine : {len(accrochees)} œuvres tirées au sort parmi celles que le "
            f"{musee['musee']} ({musee['lieu']}) expose en ce moment dans ses salles, toutes dans le domaine public, "
            f"photographiées et publiées en CC0 par le musée. Tirage de la semaine du {jour} : quand le musée change son "
            "accrochage, l'exposition change avec lui."),
        f"vitrine = {chaine(accrochees[0][1] + '.jpg')}",
        "", "[musee]", f"nom = {chaine(musee['musee'])}", f"ville = {chaine(musee['lieu'])}",
        f"architecte = {chaine(musee['architecte'])}", f"construction = {chaine(musee['construction'])}",
        f"presentation = {chaine(musee['presentation'])}",
        "ici = " + chaine(
            f"Le bâtiment d'ARTEON est une enfilade de {salles} salles claires, reliées par des couloirs, où les "
            "œuvres sont accrochées comme le musée les montre aujourd'hui : chacune garde, sur son cartel, la salle "
            "où on peut la voir en vrai."),
        "",
    ]
    for n in range(salles):
        lignes += ["[[salles]]", f"nom = {chaine(noms_salles[n])}",
                   f"theme = {chaine('Le tirage de la semaine, au ' + musee['musee'] + '.')}",
                   "largeur = 12", "profondeur = 10", "hauteur = 5", ""]
    places = [("ouest", 1), ("ouest", 3), ("est", 1), ("est", 3)]
    for i, (o, nom, ratio) in enumerate(accrochees):
        mur, place = places[i % 4]
        mesures, _ = dimensions_cm(o["mesures"])
        technique = ", ".join(t for t in (technique_fr(o["technique"]), mesures) if t)
        cartel = f"Exposée en ce moment au {musee['musee']}" + (f", {o['salle']}" if o["salle"] else "") + "."
        lignes += [
            "[[oeuvres]]", f"titre = {chaine(o['titre'][:140])}", f"artiste = {chaine(o['artiste'][:120])}",
            f"annee = {chaine(date_fr(o['date']))}", f"image = {chaine(nom + '.jpg')}",
            f"cadre = {chaine(cadre_pour(o['genre']))}", f"technique = {chaine(technique[:200])}",
            f"collection = {chaine(musee['musee'] + ', ' + musee['lieu'].split(',')[0])}",
            f"note = {chaine(cartel)}",
            "licence = \"CC0\"", f"source = {chaine(o['lien'])}",
        ]
        if o["texte"]:
            lignes.append("description = " + chaine("Le musée la présente ainsi (en anglais) : « " + o["texte"][:1200] + " »"))
        lignes += [f"salle = {i // 4 + 1}", f"mur = {chaine(mur)}", f"place = {place}",
                   f"largeur = {largeur_pour(o, ratio)}", ""]
    (RACINE / "expositions" / f"{musee['slug']}.toml").write_text("\n".join(lignes), encoding="utf-8")
    print(f"  {len(accrochees)} œuvres accrochées")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--graine", help="le tirage (par défaut : l'année et la semaine en cours)")
    args = parser.parse_args()
    aujourdhui = datetime.date.today().isocalendar()
    semaine = (aujourdhui[0], aujourdhui[1])
    graine = args.graine or f"{semaine[0]}-S{semaine[1]}"
    reussies = 0
    for musee in MUSEES:
        try:
            monter(musee, semaine, graine)
            reussies += 1
        except Exception as erreur:
            print(f"  {musee['musee']} est sauté : {erreur}", file=sys.stderr)
            (RACINE / "expositions" / f"{musee['slug']}.toml").unlink(missing_ok=True)
            shutil.rmtree(RACINE / "public" / "tableaux" / musee["slug"], ignore_errors=True)
    print(f"{reussies} exposition(s) éphémère(s) sur {len(MUSEES)}")


if __name__ == "__main__":
    main()
