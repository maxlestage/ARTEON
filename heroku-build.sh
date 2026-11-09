#!/usr/bin/env bash
# Construit ARTEON sur Heroku, une fois que le buildpack emk/rust a installé Rust (voir
# RustConfig) : les expositions éphémères de la semaine, le serveur, les îlots 3D en
# WebAssembly, puis les données des expositions à côté. Le Procfile lance ensuite
# target/release/arteon. Marche aussi à la main, depuis le dépôt : ./heroku-build.sh
set -euo pipefail
cd "$(dirname "$0")"

# Les variables de configuration de l'application (Heroku → Settings → Config Vars) :
# le buildpack les passe dans ENV_DIR.
config() {
  local name=$1 default=$2
  if [ -n "${!name:-}" ]; then echo "${!name}"
  elif [ -n "${ENV_DIR:-}" ] && [ -f "$ENV_DIR/$name" ]; then cat "$ENV_DIR/$name"
  else echo "$default"; fi
}

# Si Paquets ou ses dépôts deviennent privés : un jeton GitHub qui peut les lire, dans la
# variable REPOS_TOKEN. Il ne figure dans aucune adresse.
REPOS_TOKEN=$(config REPOS_TOKEN "")
if [ -n "$REPOS_TOKEN" ]; then
  export REPOS_TOKEN CARGO_NET_GIT_FETCH_WITH_CLI=true GIT_CONFIG_COUNT=1 \
    GIT_CONFIG_KEY_0=credential.https://github.com.helper \
    GIT_CONFIG_VALUE_0='!f() { test "$1" = get && echo username=x-access-token && echo "password=$REPOS_TOKEN"; }; f'
fi

# Cargo compile dans le cache du buildpack (CARGO_TARGET_DIR) : il est gardé d'un
# déploiement à l'autre, avec celui des expositions (arteon-cache).
target="${CARGO_TARGET_DIR:-target}"

echo "-----> Expositions éphémères de la semaine"
if [ "$(config EPHEMERES 1)" = 1 ] && command -v python3 > /dev/null; then
  timeout 300 python3 outils/ephemeres.py || echo "       pas d'exposition éphémère cette fois"
else
  echo "       désactivées (EPHEMERES=0) ou Python absent"
fi

echo "-----> Le serveur"
cargo build --release --bin arteon
mkdir -p target/release
if [ "$target" != target ]; then
  cp "$target/release/arteon" target/release/
fi

# Les données des expositions (mosaïques, sculptures, textes), écrites par build.rs dans
# le dossier de compilation : copiées dans l'application (ARTEON_DONNEES, voir Procfile).
donnees=$(ls -td "$target"/release/build/arteon-*/out/donnees | head -1)
rm -rf donnees
cp -r "$donnees" donnees
echo "       target/release/arteon et donnees/ ($(ls donnees | wc -l) fichiers) prêts"

echo "-----> Les îlots 3D (WebAssembly)"
rustup target add wasm32-unknown-unknown
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release

# wasm-bindgen doit avoir la version de Cargo.lock : un binaire publié, ou compilé s'il
# ne peut pas être téléchargé.
version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
bin="$target/wasm-bindgen-$version"
if [ ! -x "$bin/wasm-bindgen" ]; then
  mkdir -p "$bin"
  curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}/wasm-bindgen-${version}-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz --strip-components=1 -C "$bin" \
    || cargo install wasm-bindgen-cli --version "$version" --locked --root "$bin" --bin wasm-bindgen
  [ -x "$bin/wasm-bindgen" ] || mv "$bin/bin/wasm-bindgen" "$bin/"
fi
rm -rf pkg
"$bin/wasm-bindgen" --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir pkg "$target/wasm32-unknown-unknown/wasm-release/arteon.wasm"
echo "       îlots prêts dans pkg/"
