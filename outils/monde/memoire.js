// La mémoire du navigateur : les réponses de l'API ARTEON (le catalogue, les collections du
// monde, le catalogue de chaque artiste) gardées compressées (gzip) dans IndexedDB, pour que
// les visites suivantes soient immédiates. Rien ne sort du navigateur ; tout se vide avec
// `oublierTout()` ou les réglages du navigateur. Sans IndexedDB (navigation privée, vieux
// navigateur), tout marche quand même, depuis le réseau.

const BASE = "arteon", MAGASIN = "reponses";
const gzip = typeof CompressionStream !== "undefined" && typeof DecompressionStream !== "undefined";

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
  const url = new URL(adresse, location.href).href;
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
