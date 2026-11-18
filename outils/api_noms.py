#!/usr/bin/env python3
"""Les artistes dont l'API statique publie le catalogue complet (cargo run -- api).

  python3 outils/api_noms.py <site> [nombre] > noms.txt

Tous les artistes des expositions (catalogue.json), puis ceux des collections du monde
(monde/donnees/liste.json), du plus riche en œuvres au moins riche, puis tous ceux de
l'annuaire (outils/monde/annuaire.json), jusqu'à `nombre` (3 000 par défaut). Un nom par ligne. Bibliothèque standard de Python.
"""

import collections
import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    site = Path(sys.argv[1])
    nombre = int(sys.argv[2]) if len(sys.argv) > 2 else 3000
    noms = []
    catalogue = json.loads((site / "catalogue.json").read_text(encoding="utf-8"))
    for ex in catalogue["exhibitions"]:
        for w in ex["works"]:
            nom = w.get("artist") or ex.get("artist")
            if nom and nom not in noms:
                noms.append(nom)
    liste = site / "monde" / "donnees" / "liste.json"
    if liste.exists():
        compte = collections.Counter(l[1] for l in json.loads(liste.read_text(encoding="utf-8")) if l[1])
        for nom, _ in compte.most_common():
            if len(noms) >= nombre:
                break
            if nom != "Artiste inconnu" and nom not in noms:
                noms.append(nom)
    # Puis l'annuaire (outils/annuaire.py) : tous les autres artistes, des plus riches aux
    # moins riches.
    annuaire = Path(__file__).parent / "monde" / "annuaire.json"
    if annuaire.exists():
        vus = set(noms)
        for a in sorted(json.loads(annuaire.read_text(encoding="utf-8")), key=lambda a: -a[3]):
            if len(noms) >= nombre:
                break
            if a[0] not in vus:
                vus.add(a[0])
                noms.append(a[0])
    print("\n".join(noms))


if __name__ == "__main__":
    main()
