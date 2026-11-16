#!/usr/bin/env bash
# Les images et les modèles d'ARTEON gardés sur Nuage, comme outils/nuage.py mais sans
# Python : curl, sha256sum et les nuage.lock suffisent. Pour Heroku, dont les images de
# compilation et d'exécution n'ont pas toujours Python (heroku-build.sh).
#
#   bash outils/nuage.sh recuperer   # télécharge ce qui manque ou a changé
#   bash outils/nuage.sh envoyer     # envoie les fichiers des nuage.lock présents sur le disque
#   bash outils/nuage.sh etat        # 0 si Nuage a tous les fichiers des nuage.lock
#   bash outils/nuage.sh liberer     # efface les copies locales
#
# Réglages : NUAGE_URL (le serveur, celui d'ARTEON par défaut), NUAGE_TOKEN (un jeton de
# l'application ARTEON), ARTEON_NUAGE_CACHE (un dossier de cache, facultatif).
set -euo pipefail
cd "$(dirname "$0")/.."

URL="${NUAGE_URL:-}"
URL="${URL:-https://nuage-f48a8945f31c.herokuapp.com}"
export URL="${URL%/}" CACHE="${ARTEON_NUAGE_CACHE:-}"

# Chaque fichier attendu : « empreinte taille chemin ».
entrees() {
  local lock dossier h taille nom
  for lock in public/tableaux/*/nuage.lock modeles/nuage.lock; do
    [ -f "$lock" ] || continue
    case "$lock" in public/tableaux/ephemere-*) continue ;; esac
    dossier=$(dirname "$lock")
    while read -r h taille nom; do
      case "$h" in '' | '#'*) continue ;; esac
      echo "$h $taille $dossier/$nom"
    done < "$lock"
  done
}

jeton() {
  if [ -z "${NUAGE_TOKEN:-}" ]; then
    echo "nuage.sh : il faut NUAGE_TOKEN, un jeton de l'application ARTEON sur Nuage ($URL/admin → l'application → Tokens)" >&2
    exit 1
  fi
}

# Un fichier : téléchargé (s'il manque ou a changé), vérifié, gardé dans le cache.
un_fichier() {
  local h=$1 chemin=$3 nom=${3#public/}
  if [ -f "$chemin" ] && [ "$(sha256sum < "$chemin" | cut -d' ' -f1)" = "$h" ]; then return 0; fi
  mkdir -p "$(dirname "$chemin")"
  if [ -n "$CACHE" ] && [ -f "$CACHE/$h" ] && [ "$(sha256sum < "$CACHE/$h" | cut -d' ' -f1)" = "$h" ]; then
    cp "$CACHE/$h" "$chemin"
    return 0
  fi
  # GET /v1/files/<nom> répond 307 vers le bucket : curl suit, sans renvoyer le jeton.
  curl -fsSL --retry 4 --retry-all-errors -H "Authorization: Bearer $NUAGE_TOKEN" \
    -o "$chemin.partiel" "$URL/v1/files/$nom" || { rm -f "$chemin.partiel"; echo "nuage.sh : $nom introuvable sur Nuage" >&2; return 1; }
  if [ "$(sha256sum < "$chemin.partiel" | cut -d' ' -f1)" != "$h" ]; then
    rm -f "$chemin.partiel"
    echo "nuage.sh : $nom n'a pas l'empreinte de son nuage.lock" >&2
    return 1
  fi
  mv "$chemin.partiel" "$chemin"
  if [ -n "$CACHE" ]; then mkdir -p "$CACHE" && cp "$chemin" "$CACHE/$h"; fi
}

# Un fichier envoyé : un lien signé, puis les octets vers le bucket.
un_envoi() {
  local chemin=$3 nom=${3#public/} reponse lien type=application/octet-stream
  [ -f "$chemin" ] || return 0
  case "$chemin" in *.jpg | *.jpeg) type=image/jpeg ;; *.png) type=image/png ;; *.webp) type=image/webp ;; esac
  reponse=$(curl -fsS --retry 4 --retry-all-errors -X POST -H "Authorization: Bearer $NUAGE_TOKEN" "$URL/v1/files/$nom") \
    || { echo "nuage.sh : pas de lien d'envoi pour $nom" >&2; return 1; }
  lien=$(printf '%s' "$reponse" | sed -n 's/.*"url" *: *"\([^"]*\)".*/\1/p')
  [ -n "$lien" ] || { echo "nuage.sh : réponse inattendue pour $nom : $reponse" >&2; return 1; }
  curl -fsS --retry 4 --retry-all-errors -X PUT -H "Content-Type: $type" --data-binary @"$chemin" -o /dev/null "$lien" \
    || { echo "nuage.sh : envoi de $nom refusé" >&2; return 1; }
}
export -f un_fichier un_envoi

case "${1:-}" in
  recuperer)
    total=$(entrees | wc -l)
    manquants=$(entrees | while read -r h taille chemin; do
      if ! { [ -f "$chemin" ] && [ "$(stat -c %s "$chemin")" = "$taille" ]; }; then echo x; fi
    done | wc -l)
    if [ "$manquants" -gt 0 ] && { [ -z "$CACHE" ] || [ ! -d "$CACHE" ]; }; then jeton; fi
    export NUAGE_TOKEN="${NUAGE_TOKEN:-}"
    entrees | xargs -P 8 -L 1 bash -c 'un_fichier "$@"' _
    echo "       les $total fichiers gardés sur Nuage sont sur le disque"
    ;;
  envoyer)
    jeton
    export NUAGE_TOKEN
    total=$(entrees | wc -l)
    entrees | xargs -P 8 -L 1 bash -c 'un_envoi "$@"' _
    echo "       $total fichiers envoyés sur $URL"
    ;;
  etat)
    jeton
    liste=$(curl -fsS --retry 4 --retry-all-errors -H "Authorization: Bearer $NUAGE_TOKEN" "$URL/v1/files") || exit 1
    distants=$(printf '%s' "$liste" | grep -o '"name" *: *"[^"]*"' | sed 's/.*"\([^"]*\)"$/\1/' | sort -u)
    absents=$(entrees | while read -r _ _ chemin; do echo "${chemin#public/}"; done | sort -u | comm -23 - <(echo "$distants"))
    if [ -n "$absents" ]; then
      echo "absents de Nuage :" && echo "$absents" | head -20
      exit 1
    fi
    echo "Nuage a les $(entrees | wc -l) fichiers des nuage.lock"
    ;;
  liberer)
    octets=0
    while read -r h taille chemin; do
      if [ -f "$chemin" ] && [ "$(stat -c %s "$chemin")" = "$taille" ]; then
        rm -f "$chemin"
        octets=$((octets + taille))
      fi
    done < <(entrees)
    echo "       $((octets / 1000000)) Mo libérés : le serveur prend ces fichiers sur Nuage"
    ;;
  *)
    sed -n '2,16p' "$0"
    exit 2
    ;;
esac
