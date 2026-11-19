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
    d = re.sub(r"(?<![A-Za-z.])(c\.|ca\.|circa|about)\s*", "vers ", d, flags=re.I)
    d = re.sub(r"\bearly\s+", "début du ", d, flags=re.I)
    d = re.sub(r"\blate\s+", "fin du ", d, flags=re.I)
    d = re.sub(r"\bmid-?\s*", "milieu du ", d, flags=re.I)

    def siecle(m):
        n = int(m.group(1))
        return (ROMAINS[n] if n < len(ROMAINS) else f"{n}ᵉ") + " siècle"

    d = re.sub(r"(\d{1,2})(?:st|nd|rd|th)\s+century", siecle, d, flags=re.I)
    d = re.sub(r"\b(?:B\.C\.E\.|B\.C\.|BCE|BC)(?![A-Za-z])", "av. J.-C.", d)
    d = re.sub(r"\b(?:A\.D\.|AD|C\.E\.|CE)(?![A-Za-z])", "apr. J.-C.", d)
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
    for page in alea.sample(range(1, pages + 1), min(7, pages)):
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
            "vie": o.get("artist_display") or "", "date": o.get("date_display"), "technique": o.get("medium_display"), "mesures": o.get("dimensions"),
            "salle": o.get("gallery_title") or "", "genre": o.get("classification_title") or "",
            "texte": texte, "lien": f"https://www.artic.edu/artworks/{o['id']}",
            # 843 px : la seule largeur que l'IIIF de Chicago sert à coup sûr (sa taille en
            # cache) ; 1000 px y est refusé (403).
            "image": f"https://www.artic.edu/iiif/2/{o['image_id']}/full/843,/0/default.jpg",
            "entetes": {"AIC-User-Agent": AGENT},
            "ratio": vignette["height"] / vignette["width"],
        })
    return oeuvres


def cleveland(alea, combien):
    base = "https://openaccess-api.clevelandart.org/api/artworks/"
    filtre = {"cc0": "1", "has_image": "1", "currently_on_view": "1", "limit": "1"}
    total = json_de(base + "?" + urllib.parse.urlencode(filtre))["info"]["total"]
    oeuvres = []
    for depart in alea.sample(range(0, max(1, total - 40)), 5):
        lot = json_de(base + "?" + urllib.parse.urlencode({**filtre, "limit": "40", "skip": str(depart)}))
        for o in lot["data"]:
            web = (o.get("images") or {}).get("web") or {}
            if not web.get("url") or not web.get("width"):
                continue
            createurs = o.get("creators") or []
            artiste = createurs[0]["description"].split(" (")[0] if createurs else ", ".join(o.get("culture") or []) or "Artiste inconnu"
            oeuvres.append({
                "titre": o["title"], "artiste": artiste, "date": o.get("creation_date"),
                "vie": createurs[0].get("description", "") if createurs else "",
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
    # Au plus cinq minutes : quand l'API du Met ralentit ou refuse, on accroche ce qu'on a.
    fin = time.time() + 300
    for ident in alea.sample(identifiants, min(300, len(identifiants))):
        if len(oeuvres) >= combien * 4 or time.time() > fin:
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
            "vie": (o.get("artistDisplayBio") or "") + (f" died {o['artistEndDate']}" if str(o.get("artistEndDate") or "").strip("0") and o.get("artistEndDate") not in ("9999",) else ""),
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
        "en": {"titre": "Now at the Art Institute of Chicago", "lieu": "Chicago, United States",
               "presentation": (
                   "Built for the 1893 World's Columbian Exposition on Michigan Avenue and guarded by two bronze lions, the "
                   "Art Institute of Chicago is one of the largest art museums in the United States. Its galleries hold a "
                   "famous group of Impressionist and Post-Impressionist paintings (Seurat's Grande Jatte has been there "
                   "since 1926) as well as art from Asia, Africa, the Americas and Europe, arms and armour, decorative arts "
                   "and photography. In 2009 Renzo Piano's light-filled Modern Wing enlarged the museum. It freely publishes "
                   "online the images of its public-domain works, and says which are on view in its galleries.")},
        "es": {"titre": "Ahora en el Art Institute of Chicago", "lieu": "Chicago, Estados Unidos",
               "presentation": (
                   "Construido para la Exposición Universal de 1893 en Michigan Avenue y custodiado por dos leones de bronce, "
                   "el Art Institute of Chicago es uno de los mayores museos de arte de Estados Unidos. Sus salas reúnen un "
                   "célebre conjunto de pinturas impresionistas y postimpresionistas (la Grande Jatte de Seurat está allí "
                   "desde 1926), además de arte de Asia, África, las Américas y Europa, armas, artes decorativas y "
                   "fotografía. En 2009, el ala moderna de Renzo Piano, bañada de luz, amplió el museo. Publica libremente en "
                   "línea las imágenes de sus obras de dominio público e indica cuáles están expuestas en sus salas.")},
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
        "en": {"titre": "Now at the Cleveland Museum of Art", "lieu": "Cleveland, United States",
               "architecte": "Hubbell & Benes; Rafael Viñoly (expansion)",
               "presentation": (
                   "Opened in 1916 in a white-marble neoclassical temple beside Wade Park, the Cleveland Museum of Art has "
                   "always been free. Its collections span five thousand years of art, from ancient Egypt to contemporary "
                   "art, with outstanding holdings of medieval art, Asian art and European painting. In 2013 Rafael Viñoly's "
                   "expansion linked the buildings with a vast covered atrium. In 2019 the museum released the images of all "
                   "its public-domain works under CC0, and it indicates which ones are on view.")},
        "es": {"titre": "Ahora en el Museo de Arte de Cleveland", "lieu": "Cleveland, Estados Unidos",
               "architecte": "Hubbell & Benes; Rafael Viñoly (ampliación)",
               "presentation": (
                   "Inaugurado en 1916 en un templo neoclásico de mármol blanco, junto a Wade Park, el Cleveland Museum of "
                   "Art ha sido siempre gratuito. Sus colecciones abarcan cinco mil años de arte, del antiguo Egipto al arte "
                   "contemporáneo, con conjuntos notables de arte medieval, arte asiático y pintura europea. En 2013, la "
                   "ampliación de Rafael Viñoly unió los edificios mediante un gran atrio cubierto. En 2019 el museo puso "
                   "bajo licencia CC0 las imágenes de todas sus obras de dominio público, e indica cuáles están expuestas.")},
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
        "en": {"titre": "Now at the Metropolitan Museum of Art, New York", "lieu": "New York, United States",
               "architecte": "Calvert Vaux and Jacob Wrey Mould; Richard Morris Hunt (façade)",
               "presentation": (
                   "Founded in 1870, the Metropolitan Museum of Art moved in 1880 to Central Park, into a building by "
                   "Calvert Vaux and Jacob Wrey Mould, then in 1902 gained its great Beaux-Arts façade on Fifth Avenue. The "
                   "“Met” is one of the largest museums in the world: five thousand years of art from every continent, from "
                   "the Egyptian Temple of Dendur to armour, musical instruments, costumes and European paintings. Since "
                   "2017 it has offered the images of its public-domain works under CC0, and it gives the gallery where "
                   "each one is on view.")},
        "es": {"titre": "Ahora en el Metropolitan Museum de Nueva York", "lieu": "Nueva York, Estados Unidos", "ville": "Nueva York",
               "architecte": "Calvert Vaux y Jacob Wrey Mould; Richard Morris Hunt (fachada)",
               "presentation": (
                   "Fundado en 1870, el Metropolitan Museum of Art se instala en 1880 en Central Park, en un edificio de "
                   "Calvert Vaux y Jacob Wrey Mould, y en 1902 recibe su gran fachada Beaux-Arts sobre la Quinta Avenida. El "
                   "«Met» es uno de los mayores museos del mundo: cinco mil años de arte de todos los continentes, del templo "
                   "egipcio de Dendur a las armaduras, los instrumentos musicales, los trajes y la pintura europea. Desde "
                   "2017 ofrece con licencia CC0 las imágenes de sus obras de dominio público e indica la sala donde se "
                   "expone cada una.")},
    },
]


# ─── L'exposition ─────────────────────────────────────────────────────────────

def libre(o):
    """Le domaine public en Europe comme aux États-Unis : l'œuvre date d'avant 1931 et
    l'artiste est mort en 1955 ou avant (les musées publient en CC0 ce qui est libre aux
    États-Unis seulement)."""
    date = str(o.get("date") or "")
    annees = [int(a) for a in re.findall(r"(?<!\d)(1[0-9]{3}|20[0-9]{2})(?!\d)", date)]
    annees += [int(m.group(1)) // 100 * 100 + int(m.group(2))
               for m in re.finditer(r"(?<!\d)(1[0-9]{3})\s*[–-]\s*([0-9]{2})(?!\d)", date)]
    if annees and max(annees) > 1930:
        return False
    vie = str(o.get("vie") or "")
    m = re.search(r"(1[0-9]{3})\s*[–-]\s*(1[0-9]{3}|20[0-9]{2})", vie) or re.search(r"died\s*(1[0-9]{3}|20[0-9]{2})", vie)
    if m:
        return int(m.groups()[-1]) <= 1955
    m = re.search(r"(?:born|b\.)\s*(1[0-9]{3}|20[0-9]{2})", vie)
    return not (m and int(m.group(1)) > 1890)


def choisir(oeuvres, alea, combien):
    """Des œuvres variées, dans l'ordre où les accrocher : d'abord pas plus de deux par
    artiste ni de quatre du même genre, puis les autres en réserve (une image qui ne se
    télécharge pas est remplacée par la suivante). Seulement le domaine public."""
    oeuvres = [o for o in oeuvres if libre(o)]
    alea.shuffle(oeuvres)
    par_artiste, par_genre, choix, reserve = {}, {}, [], []
    for o in oeuvres:
        a, g = o["artiste"], (o["genre"] or "?").lower()
        if par_artiste.get(a, 0) >= 2 or par_genre.get(g, 0) >= 4:
            reserve.append(o)
            continue
        par_artiste[a] = par_artiste.get(a, 0) + 1
        par_genre[g] = par_genre.get(g, 0) + 1
        choix.append(o)
    return (choix + reserve)[:combien]


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


def date_es(date):
    """« c. 1890 », « 19th century », « 900–400 B.C. » en espagnol."""
    d = (date or "").strip()
    d = re.sub(r"(?<![A-Za-z.])(c\.|ca\.|circa|about)\s*", "hacia ", d, flags=re.I)
    d = re.sub(r"\bearly\s+", "principios del ", d, flags=re.I)
    d = re.sub(r"\blate\s+", "finales del ", d, flags=re.I)
    d = re.sub(r"\bmid-?\s*", "mediados del ", d, flags=re.I)
    d = re.sub(r"(\d{1,2})(?:st|nd|rd|th)\s+century",
               lambda m: "siglo " + (ROMAINS[int(m.group(1))].rstrip("ᵉʳ") if int(m.group(1)) < len(ROMAINS) else m.group(1)), d, flags=re.I)
    d = re.sub(r"\b(?:B\.C\.E\.|B\.C\.|BCE|BC)(?![A-Za-z])", "a. C.", d)
    d = re.sub(r"\b(?:A\.D\.|AD|C\.E\.|CE)(?![A-Za-z])", "d. C.", d)
    d = re.sub(r"(\d)-(\d)", r"\1–\2", d).replace(" or ", " o ")
    return d[:1].upper() + d[1:] if d else "Fecha desconocida"


# Les textes écrits par le script, dans les trois langues du site.
LANGUES = {
    "fr": {
        "mois": ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"],
        "jour": "{j} {m} {a}", "epoques": "Toutes époques", "court": "Éphémère", "expo": "Exposition éphémère",
        "artistes": "Artistes du monde entier", "semaine": "semaine du",
        "salles": ["Première salle", "Deuxième salle", "Troisième salle", "Quatrième salle"],
        "intro": "Une exposition qui change chaque semaine : {n} œuvres tirées au sort parmi celles que le {musee} ({lieu}) "
                 "expose en ce moment dans ses salles, toutes dans le domaine public, photographiées et publiées en CC0 par "
                 "le musée. Tirage de la semaine du {jour} : quand le musée change son accrochage, l'exposition change avec lui.",
        "ici": "Le bâtiment d'ARTEON est une enfilade de {n} salles claires, reliées par des couloirs, où les œuvres sont "
               "accrochées comme le musée les montre aujourd'hui : chacune garde, sur son cartel, la salle où on peut la voir en vrai.",
        "theme": "Le tirage de la semaine, au {musee}.", "cartel": "Exposée en ce moment au {musee}",
        "texte": "Le musée la présente ainsi (en anglais) :", "date": date_fr,
    },
    "en": {
        "mois": ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
        "jour": "{j} {m} {a}", "epoques": "All periods", "court": "Temporary", "expo": "Temporary exhibition",
        "artistes": "Artists from around the world", "semaine": "week of",
        "salles": ["First room", "Second room", "Third room", "Fourth room"],
        "intro": "An exhibition that changes every week: {n} works drawn at random from those the {musee} ({lieu}) is "
                 "showing in its galleries right now, all in the public domain, photographed and published under CC0 by "
                 "the museum. Draw for the week of {jour}: when the museum changes its display, the exhibition changes with it.",
        "ici": "The ARTEON building is a suite of {n} bright rooms linked by corridors, where the works hang as the museum "
               "shows them today: each label keeps the gallery where the work can be seen in person.",
        "theme": "This week's draw, at the {musee}.", "cartel": "On view now at the {musee}",
        "texte": "The museum describes it as follows:",
        "date": lambda d: re.sub(r"(\d)-(\d)", r"\1–\2", (d or "").strip()) or "Date unknown",
    },
    "es": {
        "mois": ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"],
        "jour": "{j} de {m} de {a}", "epoques": "Todas las épocas", "court": "Efímera", "expo": "Exposición efímera",
        "artistes": "Artistas de todo el mundo", "semaine": "semana del",
        "salles": ["Primera sala", "Segunda sala", "Tercera sala", "Cuarta sala"],
        "intro": "Una exposición que cambia cada semana: {n} obras elegidas al azar entre las que el {musee} ({lieu}) "
                 "expone ahora mismo en sus salas, todas de dominio público, fotografiadas y publicadas con licencia CC0 por "
                 "el museo. Sorteo de la semana del {jour}: cuando el museo cambia su montaje, la exposición cambia con él.",
        "ici": "El edificio de ARTEON es una sucesión de {n} salas luminosas, unidas por pasillos, donde las obras cuelgan "
               "como el museo las muestra hoy: cada cartela conserva la sala donde puede verse la obra en persona.",
        "theme": "El sorteo de la semana, en el {musee}.", "cartel": "Expuesta ahora en el {musee}",
        "texte": "El museo la presenta así (en inglés):", "date": date_es,
    },
}


# Les images des musées arrivent parfois très grandes (350 Ko pour une couverture de
# l'accueil) : réduites comme celles des autres expositions (1 000 px au plus, JPEG 82).
COTE_MAX, QUALITE_JPEG = 1000, 82


def alleger(image):
    """L'image réduite à COTE_MAX px de côté, si Pillow est là et qu'elle y gagne ; sinon telle quelle."""
    try:
        import io
        from PIL import Image
    except ImportError:
        return image
    try:
        with Image.open(io.BytesIO(image)) as im:
            if max(im.size) <= COTE_MAX and len(image) <= 150_000:
                return image
            im = im.convert("RGB")
            im.thumbnail((COTE_MAX, COTE_MAX), Image.LANCZOS)
            sortie = io.BytesIO()
            im.save(sortie, "JPEG", quality=QUALITE_JPEG, optimize=True, progressive=True)
            reduite = sortie.getvalue()
            return reduite if len(reduite) < len(image) else image
    except Exception:
        return image


def monter(musee, semaine, graine):
    alea = random.Random(f"{graine}-{musee['slug']}")
    print(f"• {musee['musee']}")
    # Trois fois plus de candidates que de places : on accroche les douze premières
    # dont l'image arrive.
    oeuvres = choisir(musee["source"](alea, PAR_EXPOSITION), alea, PAR_EXPOSITION * 3)
    dossier = RACINE / "public" / "tableaux" / musee["slug"]
    shutil.rmtree(dossier, ignore_errors=True)
    dossier.mkdir(parents=True)
    pris, accrochees = set(), []
    fin = time.time() + 360  # six minutes d'images au plus par musée
    for o in oeuvres:
        if len(accrochees) == PAR_EXPOSITION or time.time() > fin:
            break
        try:
            image = lire(o["image"], o["entetes"])
        except Exception as erreur:
            print(f"  image introuvable : {o['titre']} ({erreur})", file=sys.stderr)
            continue
        image = alleger(image)
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
    annees = [int(a) for o, _, _ in accrochees for a in re.findall(r"\b(\d{4})\b", o["date"] or "")]
    salles = (len(accrochees) + 3) // 4
    places = [("ouest", 1), ("ouest", 3), ("est", 1), ("est", 3)]
    # Le même fichier dans les trois langues du site : le français dans expositions/, l'anglais
    # et l'espagnol dans expositions/traductions/ (seuls les textes changent, voir
    # outils/traductions.py). Les titres et les descriptions viennent du musée, en anglais.
    for langue in ("fr", "en", "es"):
        L, M = LANGUES[langue], {**musee, **musee.get(langue, {})}
        jour = L["jour"].format(j=lundi.day, m=L["mois"][lundi.month - 1], a=lundi.year)
        dates = f"{min(annees)}–{max(annees)}" if annees else L["epoques"]
        lignes = [
            "# Exposition éphémère, écrite par outils/ephemeres.py : ne pas modifier à la main.",
            "", "[exposition]", f"ordre = {musee['ordre']}", f"nom_court = {chaine(L['court'] + ' · ' + M['ville'])}",
            f"mouvement = {chaine(L['expo'] + ' · ' + M['ville'])}", "ephemere = true",
            f"cadre = {chaine('dore')}", f"couleur = {chaine(musee['couleur'])}",
            f"titre = {chaine(M['titre'])}",
            f"artiste = {chaine(L['artistes'])}", f"dates = {chaine(dates)}",
            f"accroche = {chaine(L['expo'] + ' · ' + musee['musee'] + ' · ' + L['semaine'] + ' ' + jour)}",
            "introduction = " + chaine(L["intro"].format(n=len(accrochees), musee=musee["musee"], lieu=M["lieu"], jour=jour)),
            f"vitrine = {chaine(accrochees[0][1] + '.jpg')}",
            "", "[musee]", f"nom = {chaine(musee['musee'])}", f"ville = {chaine(M['lieu'])}",
            f"architecte = {chaine(M['architecte'])}", f"construction = {chaine(musee['construction'])}",
            f"presentation = {chaine(M['presentation'])}",
            "ici = " + chaine(L["ici"].format(n=salles)),
            "",
        ]
        for n in range(salles):
            lignes += ["[[salles]]", f"nom = {chaine(L['salles'][n])}",
                       f"theme = {chaine(L['theme'].format(musee=musee['musee']))}",
                       "largeur = 12", "profondeur = 10", "hauteur = 5", ""]
        for i, (o, nom, ratio) in enumerate(accrochees):
            mur, place = places[i % 4]
            mesures, _ = dimensions_cm(o["mesures"])
            brute = re.sub(r"\s+", " ", (o["technique"] or "").strip())
            technique = ", ".join(t for t in (technique_fr(o["technique"]) if langue == "fr" else brute[:1].upper() + brute[1:], mesures) if t)
            cartel = L["cartel"].format(musee=musee["musee"]) + (f", {o['salle']}" if o["salle"] else "") + "."
            lignes += [
                "[[oeuvres]]", f"titre = {chaine(o['titre'][:140])}", f"artiste = {chaine(o['artiste'][:120])}",
                f"annee = {chaine(L['date'](o['date']))}", f"image = {chaine(nom + '.jpg')}",
                f"cadre = {chaine(cadre_pour(o['genre']))}", f"technique = {chaine(technique[:200])}",
                f"collection = {chaine(musee['musee'] + ', ' + M['lieu'].split(',')[0])}",
                f"note = {chaine(cartel)}",
                "licence = \"CC0\"", f"source = {chaine(o['lien'])}",
            ]
            if o["texte"]:
                lignes.append("description = " + chaine(L["texte"] + " « " + o["texte"][:1200] + " »"))
            lignes += [f"salle = {i // 4 + 1}", f"mur = {chaine(mur)}", f"place = {place}",
                       f"largeur = {largeur_pour(o, ratio)}", ""]
        dossier_toml = RACINE / "expositions" if langue == "fr" else RACINE / "expositions" / "traductions" / langue
        dossier_toml.mkdir(parents=True, exist_ok=True)
        (dossier_toml / f"{musee['slug']}.toml").write_text("\n".join(lignes), encoding="utf-8")
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
            for langue in ("en", "es"):
                (RACINE / "expositions" / "traductions" / langue / f"{musee['slug']}.toml").unlink(missing_ok=True)
            shutil.rmtree(RACINE / "public" / "tableaux" / musee["slug"], ignore_errors=True)
    print(f"{reussies} exposition(s) éphémère(s) sur {len(MUSEES)}")


if __name__ == "__main__":
    main()
