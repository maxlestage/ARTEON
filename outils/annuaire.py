#!/usr/bin/env python3
"""L'annuaire de tous les artistes : chaque artiste dont au moins une œuvre (peinture, dessin,
estampe, photographie) est reproduite dans Wikidata, et dont l'œuvre est du domaine public
(mort en 1955 ou avant ; sans date de mort, né en 1890 ou avant).

  python3 outils/annuaire.py [sortie] [--minutes N]

Écrit outils/monde/annuaire.json : [[nom, naissance, mort, œuvres], …], trié par nom. Le site
et l'application le réunissent avec leurs propres artistes ; le catalogue de chacun vient
ensuite de l'API (src/api.rs), en direct des musées. Bibliothèque standard de Python.
"""

import json
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

SPARQL = "https://query-main.wikidata.org/sparql"
AGENT = "ARTEON/3.0 (musee en ligne ; annuaire des artistes)"
# Les mêmes types que la source « wikidata » de outils/monde/sources.json.
TYPES = ["Q3305213", "Q93184", "Q11060274", "Q125191", "Q18761202", "Q15123870"]
DERNIERE_MORT, DERNIERE_NAISSANCE = 1955, 1890
LOT = 400


def demander(requete, delai=120):
    """Le résultat d'une requête SPARQL (les lignes), avec trois essais."""
    corps = urllib.parse.urlencode({"query": requete}).encode()
    for essai in range(3):
        try:
            r = urllib.request.Request(SPARQL, data=corps, headers={
                "Accept": "application/sparql-results+json", "User-Agent": AGENT,
                "Content-Type": "application/x-www-form-urlencoded"})
            with urllib.request.urlopen(r, timeout=delai) as reponse:
                return json.load(reponse)["results"]["bindings"]
        except Exception as erreur:  # 429, 5xx, délai : on attend et on recommence
            attente = 20 * (essai + 1)
            print(f"  … {erreur} ; nouvel essai dans {attente} s", file=sys.stderr)
            time.sleep(attente)
    return None


def main():
    arguments = sys.argv[1:]
    minutes = 40.0
    if "--minutes" in arguments:
        i = arguments.index("--minutes")
        minutes = float(arguments[i + 1])
        del arguments[i:i + 2]
    sortie = Path(arguments[0] if arguments else Path(__file__).parent / "monde" / "annuaire.json")
    fin = time.time() + minutes * 60

    # 1. Combien d'œuvres en image pour chaque artiste, type par type.
    compte = {}
    for t in TYPES:
        lignes = demander(f"SELECT ?a (COUNT(DISTINCT ?w) AS ?n) WHERE {{ ?w wdt:P170 ?a ; wdt:P18 ?img ; wdt:P31 wd:{t} . }} GROUP BY ?a")
        if lignes is None:
            sys.exit(f"Wikidata ne répond pas pour {t} : l'annuaire n'est pas refait.")
        for l in lignes:
            q = l["a"]["value"].rsplit("/", 1)[-1]
            if q.startswith("Q"):
                compte[q] = compte.get(q, 0) + int(l["n"]["value"])
        print(f"{t} : {len(lignes)} artistes", file=sys.stderr)

    # 2. Leurs noms et leurs dates, par lots, des plus riches aux moins riches.
    ids = sorted(compte, key=lambda q: -compte[q])
    artistes = {}
    for i in range(0, len(ids), LOT):
        if time.time() > fin:
            print(f"temps écoulé : {i} artistes lus sur {len(ids)}", file=sys.stderr)
            break
        lot = " ".join(f"wd:{q}" for q in ids[i:i + LOT])
        lignes = demander(
            "SELECT ?a ?aLabel (MIN(YEAR(?n)) AS ?ne) (MIN(YEAR(?m)) AS ?mort) WHERE { "
            f"VALUES ?a {{ {lot} }} ?a wdt:P31 wd:Q5 . "
            "OPTIONAL { ?a wdt:P569 ?n } OPTIONAL { ?a wdt:P570 ?m } "
            'SERVICE wikibase:label { bd:serviceParam wikibase:language "fr,en,nl,de,it,es". } '
            "} GROUP BY ?a ?aLabel", delai=90)
        for l in lignes or []:
            q = l["a"]["value"].rsplit("/", 1)[-1]
            nom = l.get("aLabel", {}).get("value", "")
            ne = int(l["ne"]["value"]) if "ne" in l else None
            mort = int(l["mort"]["value"]) if "mort" in l else None
            if not nom or nom == q or nom.startswith("http"):
                continue
            libre = mort <= DERNIERE_MORT if mort is not None else (ne is not None and ne <= DERNIERE_NAISSANCE)
            if libre:
                artistes[q] = [nom, ne, mort, compte[q]]
        if (i // LOT) % 20 == 0:
            print(f"{i + LOT} / {len(ids)} : {len(artistes)} artistes du domaine public", file=sys.stderr)
        time.sleep(0.5)

    if len(artistes) < 1000:
        sys.exit("Trop peu d'artistes : l'annuaire n'est pas refait.")
    # Deux artistes du même nom : on garde le plus riche en œuvres.
    par_nom = {}
    for a in sorted(artistes.values(), key=lambda a: -a[3]):
        par_nom.setdefault(a[0], a)
    liste = sorted(par_nom.values(), key=lambda a: a[0].casefold())
    sortie.write_text(json.dumps(liste, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print(f"{len(liste)} artistes, {sum(a[3] for a in liste)} œuvres → {sortie}")


if __name__ == "__main__":
    main()
