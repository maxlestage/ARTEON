#!/usr/bin/env python3
"""Toutes les catégories de Wikidata dans outils/monde/sujets.json.

Les styles et mouvements, les genres, les techniques, les sujets représentés et les musées
(les collections) : pour chaque famille, toutes les valeurs que Wikidata donne à au moins
SEUIL œuvres libres de droits (en deux dimensions, avec une image, d'un artiste mort en 1955
ou avant), avec leur nom en français, en anglais et en espagnol, leur nombre d'œuvres et une
image. Les sujets choisis à la main (ceux qui réunissent plusieurs éléments de Wikidata, avec
des noms soignés) restent en tête de leur famille ; les autres s'ajoutent après eux.

Les comptes viennent de QLever (qlever.dev, un miroir public de Wikidata à l'université de
Fribourg-en-Brisgau) : le service officiel abandonne ces requêtes, trop longues pour lui.

  python3 outils/sujets_wikidata.py            met à jour outils/monde/sujets.json
  python3 outils/sujets_wikidata.py --essai    affiche les comptes sans rien écrire

Bibliothèque standard de Python.
"""

import json
import re
import sys
import time
import unicodedata
import urllib.parse
import urllib.request
from pathlib import Path

ICI = Path(__file__).resolve().parent
FICHIER = ICI / "monde" / "sujets.json"
SEUIL = 12
# Les sujets de chaque famille qui gardent une image (les premiers, que la page montre d'abord).
AVEC_IMAGE = 48
QLEVER = "https://qlever.dev/api/wikidata?query="
AGENT = "ARTEON/1.0 (musee virtuel ; https://github.com/maxlestage/ARTEON)"

PREFIXES = ("PREFIX wd: <http://www.wikidata.org/entity/> PREFIX wdt: <http://www.wikidata.org/prop/direct/> "
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> ")
# Les mêmes types d'œuvres que la requête des sujets (sujets.json, « _requete ») : des œuvres en
# deux dimensions, dont la reproduction est du domaine public comme elles.
TYPES2D = ("wd:Q3305213 wd:Q93184 wd:Q11060274 wd:Q125191 wd:Q18761202 wd:Q15123870 wd:Q18218093 wd:Q11835431 "
           "wd:Q12043905 wd:Q21647744 wd:Q18219090 wd:Q18396864 wd:Q89503149 wd:Q22669139 wd:Q282129 wd:Q21281546 "
           "wd:Q23657281 wd:Q48498 wd:Q132137 wd:Q22669850 wd:Q5078274 wd:Q56055236 wd:Q22669539")
BASE = ("VALUES ?type { " + TYPES2D + " } ?o wdt:P31 ?type ; wdt:P18 ?i ; wdt:P170 ?a . "
        "?a wdt:P570 ?d . FILTER(YEAR(?d) <= 1955) ")

# Chaque famille : comment une œuvre s'y rattache, pour les compter (ici) et pour chercher les
# œuvres d'un sujet (le motif « wikidata » de sujets.json, où {id} devient ses éléments).
FAMILLES = {
    "styles": {
        "compte": "{ ?o wdt:P135 ?x } UNION { ?a wdt:P135 ?x }",
        "wikidata": "VALUES ?s { {id} } { ?oeuvre wdt:P135 ?s } UNION { ?oeuvre wdt:P170 ?c . ?c wdt:P135 ?s }",
        "noms": ("Styles et mouvements", "Styles and movements", "Estilos y movimientos"),
    },
    "genres": {
        "compte": "?o wdt:P136 ?x .",
        "wikidata": "VALUES ?s { {id} } ?oeuvre wdt:P136 ?s .",
        "noms": ("Genres", "Genres", "Géneros"),
    },
    "techniques": {
        "compte": "{ ?o wdt:P31 ?x } UNION { ?o wdt:P186 ?x }",
        "wikidata": "VALUES ?s { {id} } { ?oeuvre wdt:P31 ?s } UNION { ?oeuvre wdt:P186 ?s }",
        "noms": ("Techniques", "Techniques", "Técnicas"),
    },
    "motifs": {
        "compte": "{ ?o wdt:P180 ?x } UNION { ?o wdt:P921 ?x }",
        "wikidata": "VALUES ?s { {id} } { ?oeuvre wdt:P180 ?s } UNION { ?oeuvre wdt:P921 ?s }",
        "noms": ("Sujets et motifs", "Subjects and motifs", "Temas y motivos"),
    },
    "collections": {
        "compte": "?o wdt:P195 ?x .",
        "wikidata": "VALUES ?s { {id} } ?oeuvre wdt:P195 ?s .",
        "noms": ("Musées et collections", "Museums and collections", "Museos y colecciones"),
    },
}

# Des valeurs de Wikidata qui ne sont pas des catégories pour un visiteur.
ECARTER = re.compile(r"witkacy|^Q\d+$|^(être humain|œuvre d’art|œuvre d'art|human|work of art)$", re.I)


def demander(sparql):
    requete = urllib.request.Request(QLEVER + urllib.parse.quote(PREFIXES + sparql),
                                     headers={"User-Agent": AGENT, "Accept": "application/sparql-results+json"})
    for essai in range(4):
        try:
            with urllib.request.urlopen(requete, timeout=300) as reponse:
                return json.loads(reponse.read())["results"]["bindings"]
        except Exception as erreur:  # QLever limite le rythme (429) : on attend
            if essai == 3:
                raise
            print(f"  QLever : {erreur} ; nouvel essai dans {20 * (essai + 1)} s", file=sys.stderr)
            time.sleep(20 * (essai + 1))


def vignette(image):
    """L'image de Wikimedia Commons en 250 px (une taille toujours prête)."""
    if not image:
        return ""
    nom = image.split("Special:FilePath/")[-1]
    return f"https://commons.wikimedia.org/wiki/Special:FilePath/{nom}?width=250"


def cle_de(texte):
    texte = unicodedata.normalize("NFKD", texte.replace("œ", "oe").replace("Œ", "oe").replace("æ", "ae"))
    texte = "".join(c for c in texte if not unicodedata.combining(c)).lower()
    return re.sub(r"[^a-z0-9]+", "-", texte).strip("-")[:60].strip("-")


def decouvrir(famille, motif):
    """Les valeurs de la famille : {qid: (n, image, fr, en, es)}."""
    lignes = demander(
        "SELECT ?x ?n ?img ?fr ?en ?es WHERE { { SELECT ?x (COUNT(DISTINCT ?o) AS ?n) (SAMPLE(?i) AS ?img) WHERE { "
        + BASE + motif + " } GROUP BY ?x HAVING(COUNT(DISTINCT ?o) >= " + str(SEUIL) + ") } "
        'OPTIONAL { ?x rdfs:label ?fr FILTER(LANG(?fr) = "fr") } OPTIONAL { ?x rdfs:label ?en FILTER(LANG(?en) = "en") } '
        'OPTIONAL { ?x rdfs:label ?es FILTER(LANG(?es) = "es") } }')
    valeurs = {}
    for ligne in lignes:
        qid = ligne["x"]["value"].rsplit("/", 1)[-1]
        if not re.fullmatch(r"Q\d+", qid) or qid in valeurs:
            continue
        texte = lambda k: ligne.get(k, {}).get("value", "").strip()
        valeurs[qid] = (int(ligne["n"]["value"]), vignette(texte("img")), texte("fr"), texte("en"), texte("es"))
    print(f"{famille} : {len(valeurs)} valeurs avec au moins {SEUIL} œuvres", file=sys.stderr)
    return valeurs


def majuscule(texte):
    return texte[:1].upper() + texte[1:] if texte else texte


def main():
    table = json.loads(FICHIER.read_text(encoding="utf-8"))
    choisies = {f["cle"]: f for f in table["familles"]}
    cles = {s["cle"] for f in table["familles"] for s in f["sujets"]}
    familles = []
    for nom, famille in FAMILLES.items():
        valeurs = decouvrir(nom, famille["compte"])
        time.sleep(3)
        avant = choisies.get(nom, {})
        sujets = []
        # Les sujets choisis à la main, avec le compte de leurs éléments.
        deja = set()
        for s in avant.get("sujets", []):
            ids = [i.removeprefix("wd:") for i in s["id"].split()]
            deja.update(ids)
            n = sum(valeurs.get(i, (0,))[0] for i in ids)
            image = next((valeurs[i][1] for i in ids if i in valeurs and valeurs[i][1]), "")
            sujets.append({**{k: v for k, v in s.items() if k not in ("n", "image")}, "n": n, "image": image})
        # Toutes les autres valeurs de Wikidata, les plus riches d'abord.
        for qid, (n, image, fr, en, es) in sorted(valeurs.items(), key=lambda kv: -kv[1][0]):
            if qid in deja:
                continue
            nom_fr = majuscule(fr or en or es)
            if not nom_fr or ECARTER.search(nom_fr):
                continue
            cle = cle_de(nom_fr) or qid.lower()
            for autre in (f"{cle}-{qid.lower()}", f"{cle}-{nom}-{qid.lower()}"):
                if cle not in cles:
                    break
                cle = autre
            cles.add(cle)
            sujets.append({"cle": cle, "id": f"wd:{qid}", "fr": nom_fr, "en": majuscule(en or fr or es),
                           "es": majuscule(es or en or fr), "n": n, "image": image})
        # Une image pour les premiers seulement (ceux que la page montre d'abord) : avec les
        # adresses de 5 000 images, le fichier pesait 400 Ko compressés au lieu de 120.
        for rang, sujet in enumerate(sujets):
            if rang >= AVEC_IMAGE:
                sujet.pop("image", None)
        fr, en, es = famille["noms"]
        familles.append({"cle": nom, "fr": avant.get("fr", fr), "en": avant.get("en", en), "es": avant.get("es", es),
                         "wikidata": famille["wikidata"], "sujets": sujets})
        print(f"  → {len(sujets)} sujets ({len(avant.get('sujets', []))} choisis à la main)", file=sys.stderr)
    if "--essai" in sys.argv:
        print(json.dumps({f["cle"]: len(f["sujets"]) for f in familles}))
        return
    table["familles"] = familles
    table["_note"] = (table["_note"].split(" Toutes les catégories")[0] + " Toutes les catégories de Wikidata qui ont au moins "
                      f"{SEUIL} œuvres libres (outils/sujets_wikidata.py, {time.strftime('%d/%m/%Y')}) : « n » leur nombre "
                      "d'œuvres, « image » une de leurs œuvres ; les sujets choisis à la main d'abord.")
    FICHIER.write_text(json.dumps(table, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print(f"{sum(len(f['sujets']) for f in familles)} sujets écrits dans {FICHIER.relative_to(ICI.parent)}", file=sys.stderr)


if __name__ == "__main__":
    main()
