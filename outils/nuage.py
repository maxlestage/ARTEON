#!/usr/bin/env python3
"""Les images des tableaux et les modèles 3D d'ARTEON, gardés sur Nuage.

Le dépôt ne garde pas les fichiers lourds : chaque dossier d'images
(`public/tableaux/<exposition>/`) et le dossier `modeles/` ont un petit fichier
`nuage.lock` qui dit quels fichiers y vont (empreinte SHA-256, taille, nom). Les octets
sont dans les fichiers de l'application ARTEON sur Nuage (`/v1/files`), sous
`tableaux/<exposition>/<image>` et `modeles/<modèle>`.

    python3 outils/nuage.py recuperer   # télécharge ce qui manque, avant de compiler
    python3 outils/nuage.py envoyer     # envoie les fichiers nouveaux ou changés
    python3 outils/nuage.py etat        # compare le dépôt, le disque et Nuage
    python3 outils/nuage.py liberer     # efface les copies locales (après la compilation)
    python3 outils/nuage.py oublier public/tableaux/monet/x.jpg   # retire un fichier
    python3 outils/nuage.py ranges      # les fichiers du disque déjà sur Nuage (pour git rm)

Réglages, par variables d'environnement :

- NUAGE_URL    le serveur Nuage (par défaut celui d'ARTEON) ;
- NUAGE_TOKEN  un jeton de l'application ARTEON : en lecture seule pour recuperer et
               etat, en lecture-écriture pour envoyer et oublier ;
- ARTEON_NUAGE_CACHE  un dossier où garder les fichiers téléchargés, rangés par
               empreinte, d'une compilation à l'autre (facultatif).

Il n'utilise que la bibliothèque standard de Python.
"""

from __future__ import annotations

import concurrent.futures
import hashlib
import json
import os
import shutil
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
TABLEAUX = RACINE / "public" / "tableaux"
MODELES = RACINE / "modeles"
LOCK = "nuage.lock"
URL_PAR_DEFAUT = "https://nuage-f48a8945f31c.herokuapp.com"
EXTENSIONS = {".jpg", ".jpeg", ".png", ".webp", ".stl", ".obj", ".glb", ".gltf"}
EN_TETE = (
    "# Les fichiers de ce dossier sont gardés sur Nuage, pas dans le dépôt :\n"
    "# python3 outils/nuage.py recuperer les télécharge, envoyer les met à jour.\n"
    "# empreinte SHA-256, octets, nom\n"
)
ESSAIS = 4


def erreur(message: str) -> None:
    print(f"nuage.py : {message}", file=sys.stderr)
    sys.exit(1)


# ─── Les dossiers et leurs nuage.lock ────────────────────────────────────────


def dossiers() -> list[Path]:
    """Les dossiers dont les fichiers vont sur Nuage : un par exposition (pas les
    éphémères, refaites à chaque publication), et modeles/."""
    liste = []
    if TABLEAUX.is_dir():
        liste += sorted(
            d for d in TABLEAUX.iterdir() if d.is_dir() and not d.name.startswith("ephemere-")
        )
    if MODELES.is_dir():
        liste.append(MODELES)
    return liste


def nom_distant(chemin: Path) -> str:
    """public/tableaux/monet/x.jpg → tableaux/monet/x.jpg ; modeles/a.stl → modeles/a.stl."""
    relatif = chemin.relative_to(RACINE).as_posix()
    return relatif.removeprefix("public/")


def lire_lock(dossier: Path) -> dict[str, tuple[str, int]]:
    """Le nuage.lock d'un dossier : nom → (empreinte, taille)."""
    fichier = dossier / LOCK
    entrees = {}
    if not fichier.is_file():
        return entrees
    for numero, ligne in enumerate(fichier.read_text(encoding="utf-8").splitlines(), 1):
        ligne = ligne.strip()
        if not ligne or ligne.startswith("#"):
            continue
        parties = ligne.split(maxsplit=2)
        if len(parties) != 3 or len(parties[0]) != 64 or not parties[1].isdigit():
            erreur(f"{fichier.relative_to(RACINE)}, ligne {numero} : « {ligne} » n'est pas « empreinte octets nom »")
        entrees[parties[2]] = (parties[0], int(parties[1]))
    return entrees


def ecrire_lock(dossier: Path, entrees: dict[str, tuple[str, int]]) -> None:
    fichier = dossier / LOCK
    if not entrees:
        fichier.unlink(missing_ok=True)
        return
    lignes = [f"{h} {taille} {nom}\n" for nom, (h, taille) in sorted(entrees.items())]
    fichier.write_text(EN_TETE + "".join(lignes), encoding="utf-8")


def tous_les_locks() -> list[tuple[Path, str, str, int]]:
    """Chaque fichier attendu : (chemin local, nom sur Nuage, empreinte, taille)."""
    attendus = []
    for dossier in dossiers():
        for nom, (h, taille) in lire_lock(dossier).items():
            chemin = dossier / nom
            attendus.append((chemin, nom_distant(chemin), h, taille))
    return attendus


def empreinte(chemin: Path) -> str:
    h = hashlib.sha256()
    with open(chemin, "rb") as f:
        for bloc in iter(lambda: f.read(1 << 20), b""):
            h.update(bloc)
    return h.hexdigest()


def a_jour(chemin: Path, h: str, taille: int) -> bool:
    return chemin.is_file() and chemin.stat().st_size == taille and empreinte(chemin) == h


def nom_valide(nom: str) -> bool:
    """Un nom de fichier que Nuage accepte : lettres, chiffres, . _ - en parties séparées par /."""
    permis = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-")
    parties = nom.split("/")
    return all(p and p not in (".", "..") and set(p) <= permis for p in parties) and len(nom) <= 512


# ─── Nuage ───────────────────────────────────────────────────────────────────


class Nuage:
    def __init__(self, ecriture: bool = False):
        self.url = os.environ.get("NUAGE_URL", "").strip().rstrip("/") or URL_PAR_DEFAUT
        self.jeton = os.environ.get("NUAGE_TOKEN", "").strip()
        if not self.jeton:
            droits = "en lecture-écriture" if ecriture else "(la lecture seule suffit)"
            erreur(
                f"il faut NUAGE_TOKEN, un jeton de l'application ARTEON sur Nuage {droits} : "
                f"console de Nuage ({self.url}/admin) → l'application → Tokens"
            )

    def _requete(self, methode: str, chemin: str, *, ok=(200,)) -> dict:
        url = f"{self.url}{chemin}"
        for essai in range(ESSAIS):
            req = urllib.request.Request(url, method=methode)
            req.add_header("Authorization", f"Bearer {self.jeton}")
            req.add_header("Accept", "application/json")
            try:
                with urllib.request.urlopen(req, timeout=60) as rep:
                    corps = rep.read()
                    return json.loads(corps) if corps else {}
            except urllib.error.HTTPError as e:
                corps = e.read().decode("utf-8", "replace")
                try:
                    message = json.loads(corps)["error"]["message"]
                except (ValueError, KeyError, TypeError):
                    message = corps[:300]
                if e.code < 500 or essai == ESSAIS - 1:
                    erreur(f"{methode} {chemin} : {e.code} {message}")
            except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
                if essai == ESSAIS - 1:
                    erreur(f"{methode} {chemin} : {e}")
            time.sleep(2 ** essai)
        raise AssertionError

    def liste(self, prefixe: str) -> dict[str, int]:
        """Les fichiers de l'application sous `prefixe` : nom → taille."""
        rep = self._requete("GET", "/v1/files?" + urllib.parse.urlencode({"prefix": prefixe}))
        return {f["name"]: int(f["size"]) for f in rep.get("files", [])}

    def lien(self, nom: str) -> str:
        return self._requete("GET", f"/v1/files/{urllib.parse.quote(nom)}?link")["url"]

    def telecharger(self, nom: str, destination: Path) -> None:
        """Les octets vont du bucket au disque, sans passer par le serveur (ni par le jeton)."""
        for essai in range(ESSAIS):
            url = self.lien(nom)
            provisoire = destination.with_name(destination.name + ".partiel")
            try:
                with urllib.request.urlopen(url, timeout=120) as rep, open(provisoire, "wb") as f:
                    shutil.copyfileobj(rep, f, 1 << 20)
                provisoire.replace(destination)
                return
            except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
                provisoire.unlink(missing_ok=True)
                if essai == ESSAIS - 1:
                    erreur(f"téléchargement de {nom} : {e}")
                time.sleep(2 ** essai)

    def envoyer(self, nom: str, chemin: Path) -> None:
        donnees = chemin.read_bytes()
        for essai in range(ESSAIS):
            lien = self._requete("POST", f"/v1/files/{urllib.parse.quote(nom)}")
            req = urllib.request.Request(lien["url"], data=donnees, method=lien.get("method", "PUT"))
            req.add_header("Content-Type", type_mime(chemin))
            try:
                with urllib.request.urlopen(req, timeout=300):
                    return
            except urllib.error.HTTPError as e:
                if e.code < 500 or essai == ESSAIS - 1:
                    erreur(f"envoi de {nom} : {e.code} {e.read()[:300]!r}")
            except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
                if essai == ESSAIS - 1:
                    erreur(f"envoi de {nom} : {e}")
            time.sleep(2 ** essai)

    def supprimer(self, nom: str) -> None:
        self._requete("DELETE", f"/v1/files/{urllib.parse.quote(nom)}")


def type_mime(chemin: Path) -> str:
    return {
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".png": "image/png",
        ".webp": "image/webp",
        ".glb": "model/gltf-binary",
        ".gltf": "model/gltf+json",
    }.get(chemin.suffix.lower(), "application/octet-stream")


def en_parallele(travail, elements, titre: str) -> None:
    elements = list(elements)
    if not elements:
        return
    fait = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        for _ in pool.map(travail, elements):
            fait += 1
            if fait % 50 == 0 or fait == len(elements):
                print(f"       {titre} : {fait}/{len(elements)}", flush=True)


# ─── Les commandes ───────────────────────────────────────────────────────────


def recuperer() -> None:
    """Télécharge les fichiers des nuage.lock qui manquent sur le disque, ou qui ont changé."""
    attendus = tous_les_locks()
    cache = os.environ.get("ARTEON_NUAGE_CACHE", "").strip()
    cache = Path(cache) if cache else None
    if cache:
        cache.mkdir(parents=True, exist_ok=True)
    manquants = []
    depuis_le_cache = 0
    for chemin, nom, h, taille in attendus:
        if a_jour(chemin, h, taille):
            continue
        copie = cache / h if cache else None
        if copie and copie.is_file() and copie.stat().st_size == taille and empreinte(copie) == h:
            chemin.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(copie, chemin)
            depuis_le_cache += 1
            continue
        manquants.append((chemin, nom, h, taille))
    if depuis_le_cache:
        print(f"       {depuis_le_cache} fichiers repris du cache ({cache})")
    if not manquants:
        print(f"       les {len(attendus)} fichiers gardés sur Nuage sont sur le disque")
        return
    nuage = Nuage()
    print(f"       {len(manquants)} fichiers à télécharger depuis {nuage.url}", flush=True)

    def un(element):
        chemin, nom, h, taille = element
        chemin.parent.mkdir(parents=True, exist_ok=True)
        nuage.telecharger(nom, chemin)
        recu = empreinte(chemin)
        if recu != h:
            chemin.unlink(missing_ok=True)
            erreur(f"{nom} : Nuage a donné un fichier d'empreinte {recu[:12]}…, le nuage.lock attend {h[:12]}…")
        if cache:
            shutil.copyfile(chemin, cache / h)

    en_parallele(un, manquants, "téléchargés")


def envoyer() -> None:
    """Envoie sur Nuage les fichiers du disque absents des nuage.lock ou changés, et
    met les nuage.lock à jour. Un fichier qui n'est pas sur le disque garde sa ligne."""
    nuage = Nuage(ecriture=True)
    distants = nuage.liste("")
    a_envoyer = []
    locks = {}
    for dossier in dossiers():
        entrees = lire_lock(dossier)
        locks[dossier] = entrees
        for chemin in sorted(dossier.iterdir()):
            if not chemin.is_file() or chemin.suffix.lower() not in EXTENSIONS:
                continue
            nom = nom_distant(chemin)
            if not nom_valide(nom):
                erreur(f"{chemin.relative_to(RACINE)} : un nom de fichier n'a que des lettres sans accent, des chiffres, . _ et -")
            h, taille = empreinte(chemin), chemin.stat().st_size
            if entrees.get(chemin.name) == (h, taille) and distants.get(nom) == taille:
                continue
            a_envoyer.append((dossier, chemin, nom, h, taille))
    if not a_envoyer:
        print("       rien de nouveau : Nuage a déjà tous les fichiers")
        return
    print(f"       {len(a_envoyer)} fichiers à envoyer sur {nuage.url}", flush=True)

    def un(element):
        _, chemin, nom, _, _ = element
        nuage.envoyer(nom, chemin)

    en_parallele(un, a_envoyer, "envoyés")
    for dossier, chemin, _, h, taille in a_envoyer:
        locks[dossier][chemin.name] = (h, taille)
    for dossier, entrees in locks.items():
        ecrire_lock(dossier, entrees)
    print("       nuage.lock à jour : à valider avec git")


def etat() -> int:
    """Ce qui manque : sur Nuage (à envoyer), sur le disque (à récupérer)."""
    attendus = tous_les_locks()
    absents_disque = [c for c, _, h, t in attendus if not a_jour(c, h, t)]
    print(f"{len(attendus)} fichiers dans les nuage.lock, {len(attendus) - len(absents_disque)} sur le disque")
    nouveaux = []
    for dossier in dossiers():
        connus = lire_lock(dossier)
        for chemin in sorted(dossier.iterdir()):
            if chemin.is_file() and chemin.suffix.lower() in EXTENSIONS and chemin.name not in connus:
                nouveaux.append(chemin)
    for chemin in nouveaux:
        print(f"  à envoyer : {chemin.relative_to(RACINE)}")
    absents_nuage = []
    if os.environ.get("NUAGE_TOKEN", "").strip():
        distants = Nuage().liste("")
        absents_nuage = [n for _, n, _, t in attendus if distants.get(n) != t]
        for nom in absents_nuage:
            print(f"  absent de Nuage (ou d'une autre taille) : {nom}")
        print(f"Nuage : {len(distants)} fichiers, {sum(distants.values()) / 1e6:.1f} Mo")
    else:
        print("Nuage : pas vérifié (sans NUAGE_TOKEN)")
    return 1 if nouveaux or absents_nuage else 0


def liberer() -> None:
    """Efface les copies locales des fichiers gardés sur Nuage (le serveur les y reprend)."""
    efface = 0
    for chemin, _, h, taille in tous_les_locks():
        if chemin.is_file() and chemin.stat().st_size == taille:
            chemin.unlink()
            efface += taille
    print(f"       {efface / 1e6:.1f} Mo libérés : le serveur prend ces fichiers sur Nuage")


def ranges() -> None:
    """Les fichiers du disque qui sont dans leur nuage.lock, tels quels : ceux que le dépôt
    peut cesser de garder (le workflow « Ranger sur Nuage » en fait un git rm --cached)."""
    for chemin, _, h, taille in tous_les_locks():
        if a_jour(chemin, h, taille):
            print(chemin.relative_to(RACINE).as_posix())


def oublier(chemins: list[str]) -> None:
    """Retire des fichiers : de leur nuage.lock, de Nuage et du disque."""
    nuage = Nuage(ecriture=True)
    for texte in chemins:
        chemin = (RACINE / texte).resolve()
        dossier = chemin.parent
        entrees = lire_lock(dossier)
        if chemin.name not in entrees:
            erreur(f"{texte} n'est dans aucun nuage.lock")
        nuage.supprimer(nom_distant(chemin))
        del entrees[chemin.name]
        ecrire_lock(dossier, entrees)
        chemin.unlink(missing_ok=True)
        print(f"       {texte} retiré")


def main() -> None:
    commande, *reste = sys.argv[1:] or ["aide"]
    if commande == "recuperer":
        recuperer()
    elif commande == "envoyer":
        envoyer()
    elif commande == "etat":
        sys.exit(etat())
    elif commande == "liberer":
        liberer()
    elif commande == "ranges":
        ranges()
    elif commande == "oublier" and reste:
        oublier(reste)
    else:
        print(__doc__)
        sys.exit(0 if commande in ("aide", "-h", "--help") else 2)


if __name__ == "__main__":
    main()
