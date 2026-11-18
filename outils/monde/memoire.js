// La mémoire du navigateur : les réponses de l'API ARTEON (le catalogue, les collections du
// monde, le catalogue de chaque artiste) gardées compressées (gzip) dans IndexedDB, pour que
// les visites suivantes soient immédiates. Rien ne sort du navigateur ; tout se vide avec
// `oublierTout()` ou les réglages du navigateur. Sans IndexedDB (navigation privée, vieux
// navigateur), tout marche quand même, depuis le réseau.

const BASE = "arteon", MAGASIN = "reponses";
const gzip = typeof CompressionStream !== "undefined" && typeof DecompressionStream !== "undefined";

// La version du site (monde/donnees/version.json, réécrit à chaque mise en ligne, toujours
// demandé au réseau). Les fichiers de données vont ensemble : la liste des œuvres, leurs
// identifiants et leurs fiches doivent venir de la même mise en ligne, sinon le titre d'une
// œuvre se retrouve sous l'image d'une autre. Chaque adresse gardée porte donc la version,
// et la mémoire est vidée quand la version change.
let versionDemandee;
export function versionDuSite() {
  versionDemandee ??= fetch(new URL("donnees/version.json", import.meta.url), { cache: "no-store" })
    .then((r) => (r.ok ? r.json() : null)).then((d) => (d && d.v ? String(d.v) : null)).catch(() => null);
  return versionDemandee;
}

let ouverture;
function ouvrir() {
  ouverture ??= new Promise((ok) => {
    try {
      const r = indexedDB.open(BASE, 1);
      r.onupgradeneeded = () => r.result.createObjectStore(MAGASIN);
      r.onsuccess = () => ok(r.result);
      r.onerror = () => ok(null);
      r.onblocked = () => ok(null);
    } catch { ok(null); }
  }).then(async (db) => {
    // Une nouvelle mise en ligne : tout ce qui était gardé est oublié.
    const version = await versionDuSite();
    if (!db || !version) return db;
    await new Promise((fini) => {
      try {
        const magasin = db.transaction(MAGASIN, "readwrite").objectStore(MAGASIN);
        const lue = magasin.get("__version");
        lue.onsuccess = () => {
          if (lue.result !== version) { magasin.clear(); magasin.put(version, "__version"); }
        };
        magasin.transaction.oncomplete = magasin.transaction.onerror = magasin.transaction.onabort = () => fini();
      } catch { fini(); }
    });
    return db;
  });
  return ouverture;
}

async function lire(cle) {
  const db = await ouvrir();
  if (!db) return null;
  return new Promise((ok) => {
    try {
      const r = db.transaction(MAGASIN).objectStore(MAGASIN).get(cle);
      r.onsuccess = () => ok(r.result || null);
      r.onerror = () => ok(null);
    } catch { ok(null); }
  });
}

async function ecrire(cle, valeur) {
  const db = await ouvrir();
  if (!db) return;
  try { db.transaction(MAGASIN, "readwrite").objectStore(MAGASIN).put(valeur, cle); } catch { /* plein ou refusé */ }
}

const compresser = (texte) => (gzip
  ? new Response(new Blob([texte]).stream().pipeThrough(new CompressionStream("gzip"))).blob()
  : Promise.resolve(new Blob([texte])));

const decompresser = (entree) => (entree.gz
  ? new Response(entree.blob.stream().pipeThrough(new DecompressionStream("gzip"))).text()
  : entree.blob.text());

/** Une valeur gardée (n'importe quel JSON), compressée. */
export async function garder(cle, valeur) {
  try { await ecrire(cle, { date: Date.now(), gz: gzip, blob: await compresser(JSON.stringify(valeur)) }); } catch { /* tant pis */ }
}

/** Une valeur gardée, si elle a moins de `duree` millisecondes. */
export async function relire(cle, duree) {
  const e = await lire(cle);
  if (!e || Date.now() - e.date > duree) return null;
  try { return JSON.parse(await decompresser(e)); } catch { return null; }
}

/** Le JSON d'une adresse : depuis la mémoire du navigateur s'il a moins de `duree`
 * millisecondes, sinon depuis le réseau (puis gardé, compressé). */
export async function jsonGarde(adresse, duree = 12 * 3600e3) {
  const complete = new URL(adresse, location.href);
  // Les fichiers du site portent la version de la mise en ligne : jamais de mélange.
  const version = complete.origin === location.origin ? await versionDuSite() : null;
  if (version) complete.searchParams.set("v", version);
  const url = complete.href;
  const deja = await relire(url, duree);
  if (deja != null) return deja;
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${r.status}`);
  const texte = await r.text();
  const valeur = JSON.parse(texte);
  compresser(texte).then((blob) => ecrire(url, { date: Date.now(), gz: gzip, blob })).catch(() => {});
  return valeur;
}

/** Vide toute la mémoire d'ARTEON dans ce navigateur. */
export function oublierTout() {
  return new Promise((ok) => {
    try {
      ouverture = null;
      const r = indexedDB.deleteDatabase(BASE);
      r.onsuccess = r.onerror = r.onblocked = () => ok();
    } catch { ok(); }
  });
}
