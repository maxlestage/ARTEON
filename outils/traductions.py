#!/usr/bin/env python3
"""Les traductions des expositions : expositions/traductions/<langue>/<nom>.toml.

Une traduction est une copie du fichier de l'exposition où seuls les textes sont traduits
(titres, notices, biographies, salles…) ; tout le reste (images, nombres, couleurs, salles,
murs, noms des artistes) est identique. build.rs la lit à la place du français pour les
pages en anglais (/en/) et en espagnol (/es/) ; sans traduction, la page reste en français.

  python3 outils/traductions.py verifier [langue] [nom…]   vérifie (toutes par défaut)
  python3 outils/traductions.py manque                      ce qui reste à traduire

Bibliothèque standard de Python (3.11+).
"""

import sys
import tomllib
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
LANGUES = ["en", "es"]

# Les champs qui se traduisent (section → champs). Tout le reste doit rester identique.
TEXTES = {
    "exposition": {"nom_court", "mouvement", "titre", "artiste", "dates", "accroche", "introduction"},
    "musee": {"nom", "ville", "architecte", "construction", "presentation", "ici"},
    "salles": {"nom", "theme"},
    "oeuvres": {"titre", "annee", "technique", "collection", "note", "description", "evocation",
                "histoire", "facture", "details", "themes", "anecdote"},
    "sculptures": {"titre", "annee", "technique", "collection", "note", "description", "evocation",
                   "histoire", "facture", "details", "themes", "anecdote"},
    "artistes": {"vie", "style", "biographie"},
}


def comparer(fr, tr, chemin, fautes, section=None):
    if isinstance(fr, dict):
        if not isinstance(tr, dict):
            fautes.append(f"{chemin} : une table est attendue")
            return
        for k in fr.keys() | tr.keys():
            if k not in tr:
                fautes.append(f"{chemin}.{k} : manque")
            elif k not in fr:
                fautes.append(f"{chemin}.{k} : en trop")
            else:
                s = section if section else k
                if section and k in TEXTES.get(section, ()):
                    texte(fr[k], tr[k], f"{chemin}.{k}", fautes)
                else:
                    comparer(fr[k], tr[k], f"{chemin}.{k}" if chemin else k, fautes, s)
    elif isinstance(fr, list):
        if not isinstance(tr, list) or len(fr) != len(tr):
            fautes.append(f"{chemin} : {len(fr)} éléments attendus")
            return
        for i, (a, b) in enumerate(zip(fr, tr)):
            comparer(a, b, f"{chemin}[{i}]", fautes, section)
    elif fr != tr:
        fautes.append(f"{chemin} : doit rester {fr!r} (trouvé {tr!r})")


def texte(fr, tr, chemin, fautes):
    if isinstance(fr, list):
        if not isinstance(tr, list) or len(fr) != len(tr):
            fautes.append(f"{chemin} : {len(fr)} éléments attendus")
            return
        for i, (a, b) in enumerate(zip(fr, tr)):
            texte(a, b, f"{chemin}[{i}]", fautes)
    elif not isinstance(tr, str) or (fr.strip() and not tr.strip()):
        fautes.append(f"{chemin} : texte vide")


def verifier(langue, nom):
    original = RACINE / "expositions" / f"{nom}.toml"
    traduit = RACINE / "expositions" / "traductions" / langue / f"{nom}.toml"
    if not traduit.exists():
        return None
    try:
        tr = tomllib.loads(traduit.read_text(encoding="utf-8"))
    except Exception as e:  # TOML invalide
        return [f"TOML invalide : {e}"]
    fautes = []
    comparer(tomllib.loads(original.read_text(encoding="utf-8")), tr, "", fautes)
    return fautes


def noms():
    return sorted(p.stem for p in (RACINE / "expositions").glob("*.toml") if not p.stem.startswith("ephemere-"))


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("verifier", "manque"):
        sys.exit(__doc__)
    if sys.argv[1] == "manque":
        for langue in LANGUES:
            reste = [n for n in noms() if not (RACINE / "expositions" / "traductions" / langue / f"{n}.toml").exists()]
            print(f"{langue} : {len(noms()) - len(reste)}/{len(noms())} traduites ; reste {' '.join(reste)}")
        return
    langues = [sys.argv[2]] if len(sys.argv) > 2 else LANGUES
    choisis = sys.argv[3:] or noms()
    mauvais = 0
    for langue in langues:
        for nom in choisis:
            fautes = verifier(langue, nom)
            if fautes is None:
                continue
            if fautes:
                mauvais += 1
                print(f"✗ {langue}/{nom}.toml")
                for f in fautes[:20]:
                    print(f"    {f}")
            else:
                print(f"✓ {langue}/{nom}.toml")
    sys.exit(1 if mauvais else 0)


if __name__ == "__main__":
    main()
