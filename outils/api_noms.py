#!/usr/bin/env python3
"""Les artistes dont l'API statique publie le catalogue complet (cargo run -- api).

  python3 outils/api_noms.py <site> [nombre] > noms.txt

Tous les artistes des expositions (catalogue.json), puis les personnes des collections du
monde (monde/donnees/liste.json) et de l'annuaire (outils/monde/annuaire.json), des plus
riches en œuvres aux moins riches, puis les attributions de musée qui ne sont pas des
personnes, jusqu'à `nombre` (3 000 par défaut). Un nom par ligne. Bibliothèque standard de Python.
"""

import collections
import json
import re
import sys
from pathlib import Path


ATTRIBUTION = re.compile(
    r"[,(\d/?]| after | style of |^after |^attributed|^workshop|^circle|^follower|^manner|^school|^unknown|^anonym"
    r"|dynasty|period|century|culture|people|kingdom|empire|probably|possibly| or ", re.I)


def personne(nom):
    """Un nom qui a la forme d'un nom de personne : deux mots au moins, sans virgule, chiffre,
    parenthèse ni mot d'attribution (« after », « dynasty », « workshop »…)."""
    return len(nom.split()) >= 2 and not ATTRIBUTION.search(nom)


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
    # Les collections du monde et l'annuaire (outils/annuaire.py). Un tiers des « artistes »
    # des collections sont des attributions de musée, pas des personnes (« China, Sui
    # dynasty (581–618) », « France, Lyon », « Étienne after Honoré Daumier ») : les musées
    # n'ont rien sous ces noms-là, et leur recherche prenait un tiers du temps de calcul.
    # D'abord donc les personnes (celles de l'annuaire, ou un nom qui en a la forme), des
    # plus riches en œuvres aux moins riches ; les attributions viennent à la fin.
    compte = collections.Counter()
    liste = site / "monde" / "donnees" / "liste.json"
    if liste.exists():
        compte.update(l[1] for l in json.loads(liste.read_text(encoding="utf-8")) if l[1] and l[1] != "Artiste inconnu")
    connus = {}
    annuaire = Path(__file__).parent / "monde" / "annuaire.json"
    if annuaire.exists():
        connus = {a[0]: a[3] for a in json.loads(annuaire.read_text(encoding="utf-8"))}
    personnes = {nom: max(n, connus.get(nom, 0)) for nom, n in compte.items() if nom in connus or personne(nom)}
    personnes.update({nom: n for nom, n in connus.items() if nom not in personnes})
    vus = set(noms)
    for nom, _ in sorted(personnes.items(), key=lambda p: (-p[1], p[0])):
        if nom not in vus:
            vus.add(nom)
            noms.append(nom)
    for nom, _ in compte.most_common():
        if nom not in vus:
            vus.add(nom)
            noms.append(nom)
    noms = noms[:max(nombre, 0)]
    print("\n".join(noms))


if __name__ == "__main__":
    main()
