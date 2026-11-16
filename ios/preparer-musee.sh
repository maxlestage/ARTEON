#!/usr/bin/env bash
# Le musée embarqué de l'application iOS : tout ce qu'il faut pour visiter ARTEON sans
# Internet, copié dans ios/ARTEON.swiftpm/Musee/ :
#
#   catalogue.json          les expositions, les œuvres et leurs notices
#   tableaux/               les images des œuvres
#   donnees/                les mosaïques et sculptures des galeries 3D, leurs textes
#   pkg/                    les îlots 3D compilés en WebAssembly
#   <exposition>/galerie/   la page de chaque galerie 3D
#
# Usage (depuis n'importe où) :
#   ios/preparer-musee.sh            compile le WebAssembly, exporte le site, copie
#   ios/preparer-musee.sh <dist>     copie un export déjà fait (BASE_PATH vide !)
#
# Il faut Rust, la cible wasm32-unknown-unknown et wasm-bindgen-cli à la version de
# Cargo.lock (voir le README à la racine). En CI : le job « musee » de ios.yml.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
out=ios/ARTEON.swiftpm/Musee
dist=${1:-}

if [ -z "$dist" ]; then
  dist=target/musee-ios
  cargo build --locked --lib --target wasm32-unknown-unknown --profile wasm-release
  wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
    --out-dir pkg target/wasm32-unknown-unknown/wasm-release/arteon.wasm
  rm -rf "$dist"
  # Servi à la racine de arteon://musee/ : pas de préfixe.
  BASE_PATH= cargo run --locked --release -- export "$dist"
fi
dist=$(cd "$dist" && pwd)
test -s "$dist/catalogue.json" || { echo "✖ $dist/catalogue.json est absent"; exit 1; }
if grep -q 'from "/ARTEON/pkg/' "$dist"/*/galerie/index.html 2>/dev/null; then
  echo "✖ l'export a un préfixe (BASE_PATH) : refaire l'export avec BASE_PATH vide"
  exit 1
fi

# On repart d'un dossier vide (sauf la note qui le garde dans le dépôt).
mkdir -p "$out"
find "$out" -mindepth 1 -maxdepth 1 ! -name LISEZMOI.md -exec rm -rf {} +

cp "$dist/catalogue.json" "$out/"
cp -R "$dist/tableaux" "$dist/donnees" "$dist/pkg" "$out/"
[ -d "$dist/icons" ] && cp -R "$dist/icons" "$out/"
for page in "$dist"/*/galerie/index.html; do
  slug=$(basename "$(dirname "$(dirname "$page")")")
  mkdir -p "$out/$slug/galerie"
  cp "$page" "$out/$slug/galerie/index.html"
done

# Tout est-il là ? Chaque œuvre a son image, chaque exposition sa galerie et ses données.
python3 - "$out" <<'PY'
import json, os, sys
out = sys.argv[1]
c = json.load(open(os.path.join(out, "catalogue.json")))
missing = []
works = 0
for ex in c["exhibitions"]:
    for f in (f'{ex["gallery"]}/index.html', f'donnees/{ex["slug"]}.bin'):
        if not os.path.isfile(os.path.join(out, f)):
            missing.append(f)
    for w in ex["works"]:
        works += 1
        if not os.path.isfile(os.path.join(out, w["image"])):
            missing.append(w["image"])
for f in ("pkg/arteon.js", "pkg/arteon_bg.wasm"):
    if not os.path.isfile(os.path.join(out, f)):
        missing.append(f)
if missing:
    print("✖ manquant dans le musée embarqué :", *missing[:20], sep="\n  ")
    sys.exit(1)
size = sum(os.path.getsize(os.path.join(d, f)) for d, _, fs in os.walk(out) for f in fs)
print(f'✓ musée embarqué : {len(c["exhibitions"])} expositions, {works} œuvres, {size / 1e6:.0f} Mo')
PY
