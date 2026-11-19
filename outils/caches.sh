#!/usr/bin/env bash
# Le ménage des caches GitHub de la construction : pour chaque famille donnée (le début de la
# clé), seul le cache le plus récent reste.
#
#   GH_TOKEN=… outils/caches.sh collections-v1- api-v1-
#
# Pourquoi : un dépôt a droit à 10 Go de cache, et celui des collections en pèse près de 5.
# Deux d'entre eux, et GitHub efface les autres caches (la compilation, les catalogues des
# artistes), que la construction suivante doit alors refaire de zéro. Une clé de cache ne se
# réécrit pas : on sauve le nouveau, puis on efface les anciens ici.
# Demande la permission `actions: write`. Une erreur n'arrête pas la construction.
set -u
depot="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY manque}"
for famille in "$@"; do
  gh cache list --repo "$depot" --key "$famille" --sort created_at --order desc --limit 100 \
    --json id,key,sizeInBytes --jq '.[1:][] | "\(.id) \(.sizeInBytes) \(.key)"' |
    while read -r id octets cle; do
      echo "efface $cle ($((octets / 1000000)) Mo)"
      gh cache delete "$id" --repo "$depot" || true
    done
done
echo "Les caches du dépôt :"
gh cache list --repo "$depot" --sort created_at --order desc --limit 100 \
  --json key,sizeInBytes,ref --jq '.[] | "  \(.sizeInBytes / 1000000 | floor) Mo  \(.key)  (\(.ref))"' || true
exit 0
