#!/usr/bin/env python3
"""Envoie un dossier dans les fichiers de l'application ARTEON sur Nuage DB.

L'application iOS n'embarque plus le musée : elle lit sur Nuage le catalogue, les images,
les galeries 3D (pages, WebAssembly, données) et les collections du monde, puis garde sur
l'appareil ce qu'elle a vu. Ce script y envoie le musée préparé par ios/preparer-musee.sh et
les collections écrites par outils/collections_monde.py :

  NUAGE_TOKEN=<jeton lecture-écriture> python3 outils/nuage_envoyer.py ios/ARTEON.swiftpm/Musee

Chaque fichier garde son chemin (`tableaux/van-gogh/iris.jpg`, `van-gogh/galerie/index.html`,
`collections/vignettes-met.bin`). Un fichier déjà sur Nuage, de même taille et de même
empreinte MD5 (l'etag du bucket), n'est pas renvoyé : seules les nouveautés partent.
Les octets vont directement au bucket, par un lien signé (voir le README de Nuage).

Bibliothèque standard de Python.
"""

import argparse
import concurrent.futures
import hashlib
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

SERVEUR = os.environ.get("NUAGE_URL", "https://nuage-f48a8945f31c.herokuapp.com").rstrip("/")
IGNORES = {"LISEZMOI.md", ".DS_Store"}


def requete(methode, url, jeton=None, corps=None, entetes=None, essais=5):
    for essai in range(essais):
        try:
            r = urllib.request.Request(url, data=corps, method=methode, headers=dict(entetes or {}))
            if jeton:
                r.add_header("Authorization", f"Bearer {jeton}")
            with urllib.request.urlopen(r, timeout=300) as reponse:
                return reponse.read()
        except urllib.error.HTTPError as erreur:
            if erreur.code in (400, 401, 403, 404) or essai + 1 == essais:
                raise
        except Exception:
            if essai + 1 == essais:
                raise
        time.sleep(2 * 2 ** essai)
    return b""


def distants(jeton):
    """Les fichiers déjà sur Nuage : nom → (taille, etag)."""
    liste = json.loads(requete("GET", f"{SERVEUR}/v1/files", jeton))
    return {f["name"]: (f["size"], (f.get("etag") or "").strip('"')) for f in liste["files"]}


def md5(fichier):
    h = hashlib.md5()
    with open(fichier, "rb") as f:
        for bloc in iter(lambda: f.read(1 << 20), b""):
            h.update(bloc)
    return h.hexdigest()


def envoyer(jeton, nom, fichier):
    lien = json.loads(requete("POST", f"{SERVEUR}/v1/files/{urllib.parse.quote(nom)}", jeton))
    with open(fichier, "rb") as f:
        donnees = f.read()
    requete(lien.get("method", "PUT"), lien["url"], corps=donnees,
            entetes={"Content-Length": str(len(donnees))})
    return len(donnees)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("dossier", help="le dossier à envoyer (ios/ARTEON.swiftpm/Musee)")
    parser.add_argument("--fils", type=int, default=8)
    parser.add_argument("--essai", action="store_true", help="dire ce qui partirait, sans rien envoyer")
    args = parser.parse_args()

    jeton = os.environ.get("NUAGE_TOKEN", "").strip()
    if not jeton:
        sys.exit("NUAGE_TOKEN est vide : il faut un jeton lecture-écriture de l'application arteon")
    racine = Path(args.dossier)
    locaux = {}
    for fichier in sorted(racine.rglob("*")):
        if fichier.is_file() and fichier.name not in IGNORES:
            nom = fichier.relative_to(racine).as_posix()
            locaux[nom] = fichier
    deja = distants(jeton)

    a_envoyer = []
    for nom, fichier in locaux.items():
        taille = fichier.stat().st_size
        if nom in deja and deja[nom][0] == taille and (not deja[nom][1] or deja[nom][1] == md5(fichier)):
            continue
        a_envoyer.append((nom, fichier, taille))
    total = sum(t for _, _, t in a_envoyer)
    print(f"{len(locaux)} fichiers, {len(a_envoyer)} à envoyer ({total / 1e6:.0f} Mo), "
          f"{len(locaux) - len(a_envoyer)} déjà sur Nuage")
    if args.essai or not a_envoyer:
        return

    debut, envoyes, echecs = time.time(), 0, []
    with concurrent.futures.ThreadPoolExecutor(args.fils) as pool:
        futurs = {pool.submit(envoyer, jeton, nom, fichier): nom for nom, fichier, _ in a_envoyer}
        for i, futur in enumerate(concurrent.futures.as_completed(futurs), 1):
            try:
                envoyes += futur.result()
            except Exception as erreur:
                echecs.append(f"{futurs[futur]} ({erreur})")
            if i % 200 == 0 or i == len(futurs):
                print(f"  {i}/{len(futurs)} fichiers, {envoyes / 1e6:.0f} Mo", flush=True)
    print(f"✓ {len(a_envoyer) - len(echecs)} fichiers envoyés sur Nuage en {(time.time() - debut) / 60:.1f} min")
    if echecs:
        print("✖ échecs :", *echecs[:20], sep="\n  ")
        sys.exit(1)


if __name__ == "__main__":
    main()
