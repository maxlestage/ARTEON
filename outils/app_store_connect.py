#!/usr/bin/env python3
"""Prépare l'envoi de l'application iOS sur TestFlight, avec la clé d'API d'App Store Connect.

Lancé par .github/workflows/ios.yml avant de signer l'application. Il lit la clé dans
l'environnement (jamais dans un fichier du dépôt) :

  ASC_KEY        le contenu du fichier .p8 (« -----BEGIN PRIVATE KEY----- … »)
  ASC_KEY_ID     l'identifiant de la clé (10 caractères)
  ASC_ISSUER_ID  l'« Issuer ID » de l'équipe

puis, avec l'API d'App Store Connect :

  1. vérifie que la clé est acceptée ;
  2. trouve l'identifiant de bundle de l'application (com.maxlestage.arteon), et le crée
     s'il n'existe pas encore ;
  3. en déduit l'identifiant de l'équipe Apple (le préfixe de l'identifiant de bundle, son
     « seedId »), dont Xcode a besoin pour signer : pas besoin de le régler à la main ;
  4. vérifie que la fiche de l'application existe dans App Store Connect. Apple ne permet
     pas de la créer par l'API : si elle manque, le script dit comment la créer (une fois,
     en deux minutes, depuis un téléphone) et l'envoi sur TestFlight attend.

Il écrit dans $GITHUB_ENV (s'il existe) EQUIPE et APP_PRETE (1 ou 0), et un résumé lisible
dans $GITHUB_STEP_SUMMARY. Bibliothèque standard de Python et la commande openssl.

  python3 outils/app_store_connect.py            (dans le workflow)
  python3 outils/app_store_connect.py --essai    (vérifie la signature, sans réseau)
"""

import base64
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

API = "https://api.appstoreconnect.apple.com/v1"
BUNDLE = "com.maxlestage.arteon"
NOM = "ARTEON"


def b64(donnees: bytes) -> str:
    return base64.urlsafe_b64encode(donnees).rstrip(b"=").decode()


def der_en_brut(der: bytes) -> bytes:
    """Une signature ECDSA en DER (ce que donne openssl) → r ‖ s sur 32 octets chacun
    (ce que demande un jeton JWT ES256)."""
    if der[0] != 0x30:
        raise ValueError("signature DER inattendue")
    i = 2 if der[1] < 0x80 else 2 + (der[1] & 0x7F)
    nombres = []
    for _ in range(2):
        if der[i] != 0x02:
            raise ValueError("entier DER attendu")
        n = der[i + 1]
        valeur = der[i + 2:i + 2 + n].lstrip(b"\x00")
        if len(valeur) > 32:
            raise ValueError("entier trop long pour P-256")
        nombres.append(valeur.rjust(32, b"\x00"))
        i += 2 + n
    return nombres[0] + nombres[1]


def jeton(cle_pem: str, cle_id: str, emetteur: str) -> str:
    """Le jeton JWT ES256 de l'API d'App Store Connect (valable 15 minutes)."""
    entete = {"alg": "ES256", "kid": cle_id, "typ": "JWT"}
    maintenant = int(time.time())
    corps = {"iss": emetteur, "iat": maintenant, "exp": maintenant + 15 * 60, "aud": "appstoreconnect-v1"}
    a_signer = f"{b64(json.dumps(entete, separators=(',', ':')).encode())}.{b64(json.dumps(corps, separators=(',', ':')).encode())}"
    # La clé dans un fichier temporaire lisible par nous seuls, effacé aussitôt.
    with tempfile.TemporaryDirectory() as dossier:
        chemin = os.path.join(dossier, "cle.p8")
        descripteur = os.open(chemin, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descripteur, "w") as f:
            f.write(cle_pem.strip() + "\n")
        der = subprocess.run(["openssl", "dgst", "-sha256", "-sign", chemin], input=a_signer.encode(),
                             capture_output=True, check=True).stdout
    return f"{a_signer}.{b64(der_en_brut(der))}"


class Refus(Exception):
    def __init__(self, code, message):
        super().__init__(f"{code} : {message}")
        self.code = code


def appeler(methode, chemin, jwt, corps=None):
    requete = urllib.request.Request(API + chemin, method=methode, headers={
        "Authorization": f"Bearer {jwt}", "Content-Type": "application/json"},
        data=json.dumps(corps).encode() if corps is not None else None)
    try:
        with urllib.request.urlopen(requete, timeout=60) as r:
            return json.loads(r.read() or b"{}")
    except urllib.error.HTTPError as e:
        try:
            erreurs = json.loads(e.read()).get("errors", [])
            message = "; ".join(f"{x.get('title', '')} — {x.get('detail', '')}" for x in erreurs)
        except Exception:
            message = e.reason
        raise Refus(e.code, message) from None


def resume(lignes):
    texte = "\n".join(lignes) + "\n"
    print(texte)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as f:
            f.write(texte)


def exporter(**valeurs):
    if os.environ.get("GITHUB_ENV"):
        with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as f:
            for k, v in valeurs.items():
                f.write(f"{k}={v}\n")


def preparer():
    cle, cle_id, emetteur = (os.environ.get(n, "").strip() for n in ("ASC_KEY", "ASC_KEY_ID", "ASC_ISSUER_ID"))
    if not (cle and cle_id and emetteur):
        resume(["### TestFlight : pas réglé", "",
                "Il manque un des secrets ASC_KEY, ASC_KEY_ID ou ASC_ISSUER_ID (GitHub → Settings → "
                "Secrets and variables → Actions)."])
        exporter(APP_PRETE=0)
        return
    if "BEGIN PRIVATE KEY" not in cle:
        resume(["### TestFlight : la clé n'est pas reconnue", "",
                "Le secret ASC_KEY doit contenir tout le fichier .p8, lignes « -----BEGIN PRIVATE KEY----- » "
                "et « -----END PRIVATE KEY----- » comprises."])
        exporter(APP_PRETE=0)
        sys.exit(1)
    jwt = jeton(cle, cle_id, emetteur)

    # 1-2. L'identifiant de bundle (et la preuve que la clé est acceptée).
    filtre = urllib.parse.urlencode({"filter[identifier]": BUNDLE, "fields[bundleIds]": "identifier,name,seedId,platform"})
    try:
        trouves = [b for b in appeler("GET", f"/bundleIds?{filtre}", jwt).get("data", [])
                   if b["attributes"].get("identifier") == BUNDLE]
    except Refus as e:
        conseil = {401: "la clé est refusée : vérifiez ASC_KEY_ID et ASC_ISSUER_ID (ils vont avec cette clé), "
                        "et que la clé n'a pas été révoquée.",
                   403: "la clé n'a pas les droits nécessaires : créez-la avec le rôle Admin "
                        "(App Store Connect → Utilisateurs et accès → Intégrations)."}.get(e.code, str(e))
        resume(["### TestFlight : App Store Connect refuse la clé", "", f"Réponse d'Apple ({e}) : {conseil}"])
        exporter(APP_PRETE=0)
        sys.exit(1)
    cree = False
    if not trouves:
        try:
            nouveau = appeler("POST", "/bundleIds", jwt, {"data": {"type": "bundleIds", "attributes": {
                "identifier": BUNDLE, "name": NOM, "platform": "IOS"}}})
            trouves = [nouveau["data"]]
            cree = True
        except Refus as e:
            resume(["### TestFlight : l'identifiant de bundle n'a pas pu être créé", "", f"Réponse d'Apple : {e}"])
            exporter(APP_PRETE=0)
            sys.exit(1)
    equipe = os.environ.get("TEAM_ID", "").strip() or trouves[0]["attributes"].get("seedId", "")
    if not equipe:
        resume(["### TestFlight : identifiant d'équipe introuvable", "",
                "Ajoutez la variable APPLE_TEAM_ID (developer.apple.com → Account → Membership details)."])
        exporter(APP_PRETE=0)
        sys.exit(1)

    # 4. La fiche de l'application dans App Store Connect.
    filtre = urllib.parse.urlencode({"filter[bundleId]": BUNDLE, "fields[apps]": "name,bundleId,sku"})
    apps = [a for a in appeler("GET", f"/apps?{filtre}", jwt).get("data", []) if a["attributes"].get("bundleId") == BUNDLE]
    lignes = ["### TestFlight", "",
              f"- Clé d'API acceptée par App Store Connect.",
              f"- Identifiant de bundle `{BUNDLE}` : {'créé à l’instant' if cree else 'présent'}.",
              f"- Équipe Apple : `{equipe}`."]
    if apps:
        lignes.append(f"- Fiche de l'application : « {apps[0]['attributes'].get('name')} » — l'envoi peut se faire.")
        resume(lignes)
        exporter(EQUIPE=equipe, APP_PRETE=1)
        return
    lignes += ["- **Fiche de l'application : absente.** Apple ne permet pas de la créer par l'API ; une fois "
               "créée, chaque construction enverra l'application sur TestFlight.", "",
               "Pour la créer (deux minutes, depuis un téléphone ou un ordinateur) : "
               "[appstoreconnect.apple.com](https://appstoreconnect.apple.com/apps) → Apps → « + » → "
               "Nouvelle app :", "",
               "| Champ | Valeur |", "|---|---|",
               "| Plateformes | iOS |",
               f"| Nom | {NOM} (ou un autre nom s'il est déjà pris sur l'App Store) |",
               "| Langue principale | Français |",
               f"| Identifiant de bundle | {BUNDLE} |",
               "| SKU | arteon |",
               "| Accès utilisateur | Accès complet |", "",
               "Puis : GitHub → Actions → « Application iOS (Swift) » → Run workflow."]
    resume(lignes)
    exporter(EQUIPE=equipe, APP_PRETE=0)
    print("::warning title=TestFlight::La fiche de l'application ARTEON manque dans App Store Connect : "
          "voir le résumé du run pour la créer.")


def essai():
    """Vérifie, sans réseau, qu'un jeton signé par openssl se vérifie avec la clé publique."""
    with tempfile.TemporaryDirectory() as dossier:
        cle = os.path.join(dossier, "cle.pem")
        subprocess.run(["openssl", "genpkey", "-algorithm", "EC", "-pkeyopt", "ec_paramgen_curve:P-256", "-out", cle],
                       check=True, capture_output=True)
        pem = open(cle).read()
        jwt = jeton(pem, "ABCDE12345", "00000000-0000-0000-0000-000000000000")
        entete, corps, signature = jwt.split(".")
        brut = base64.urlsafe_b64decode(signature + "=" * (-len(signature) % 4))
        assert len(brut) == 64, len(brut)
        # r ‖ s → DER, puis vérification par openssl avec la clé publique.
        def entier(x):
            x = x.lstrip(b"\x00") or b"\x00"
            if x[0] & 0x80:
                x = b"\x00" + x
            return b"\x02" + bytes([len(x)]) + x
        der = entier(brut[:32]) + entier(brut[32:])
        der = b"\x30" + bytes([len(der)]) + der
        publique = os.path.join(dossier, "publique.pem")
        subprocess.run(["openssl", "pkey", "-in", cle, "-pubout", "-out", publique], check=True, capture_output=True)
        sig = os.path.join(dossier, "sig.der")
        open(sig, "wb").write(der)
        r = subprocess.run(["openssl", "dgst", "-sha256", "-verify", publique, "-signature", sig],
                           input=f"{entete}.{corps}".encode(), capture_output=True)
        assert r.returncode == 0, r.stdout + r.stderr
        contenu = json.loads(base64.urlsafe_b64decode(corps + "=" * (-len(corps) % 4)))
        assert contenu["aud"] == "appstoreconnect-v1" and contenu["exp"] - contenu["iat"] == 900
        assert json.loads(base64.urlsafe_b64decode(entete + "=" * (-len(entete) % 4)))["alg"] == "ES256"
    print("jeton ES256 vérifié")


if __name__ == "__main__":
    essai() if "--essai" in sys.argv else preparer()
