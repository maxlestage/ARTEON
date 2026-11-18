// Toutes les œuvres d'un artiste, en direct des musées : d'abord l'API ARTEON (le serveur
// Paquets, src/api.rs), et si elle ne répond pas, les API des musées elles-mêmes depuis le
// navigateur (Art Institute of Chicago, Cleveland Museum of Art, Metropolitan Museum).
// Rien n'est stocké sur le site. Mêmes règles que src/api.rs : domaine public en Europe
// comme aux États-Unis (artiste mort en 1955 ou avant, œuvre d'avant 1931).

import { plier } from "./commun.js";

/** L'adresse du serveur ARTEON (Heroku). */
export const API = "https://arteon-1ed4594ee106.herokuapp.com";

const DERNIERE_ANNEE = 1930, DERNIERE_MORT = 1955, MET_MAX = 40;

const annees = (t) => [...String(t ?? "").matchAll(/(?<!\d)(\d{4})(?!\d)/g)].map((m) => Number(m[1])).filter((a) => a >= 1000 && a <= 2100);

function libre(date, fin, mort, ne) {
  if ([...annees(date), ...(fin != null ? [fin] : [])].some((a) => a > DERNIERE_ANNEE)) return false;
  if (mort != null) return mort <= DERNIERE_MORT;
  if (ne != null) return ne <= 1890;
  return true;
}

function vie(t) {
  const a = annees(t), s = String(t ?? "");
  if (!a.length) return [null, null];
  if (a.length === 1) return /born|[–-]\s*$/.test(s) ? [a[0], null] : [null, a[0]];
  return [a[0], a[1]];
}

const sec = async (url, entetes = {}) => {
  const r = await fetch(url, { headers: entetes });
  if (!r.ok) throw new Error(`${r.status}`);
  return r.json();
};

/** Toutes les œuvres de l'artiste dans les musées, en direct, avec la clé de nom du site. */
export async function oeuvresEnDirect(nom, cleNom, alias = {}) {
  // L'API ARTEON d'abord (avec sa mémoire d'un jour) ; 12 secondes au plus.
  try {
    const controle = new AbortController();
    const minuterie = setTimeout(() => controle.abort(), 12000);
    const r = await fetch(`${API}/api/artiste?nom=${encodeURIComponent(nom)}`, { signal: controle.signal });
    clearTimeout(minuterie);
    if (r.ok) return { ...(await r.json()), source: "api" };
  } catch { /* le serveur dort : on demande aux musées */ }

  const cle = cleNom(nom);
  const noms = [nom, ...Object.entries(alias).filter(([, c]) => c === cle).map(([v]) => v)];
  const meme = (trouve) => {
    const t = cleNom(trouve), p = plier(trouve);
    if (t === cle) return true;
    if (/after |^follower|imitator/.test(p)) return false;
    const mots = cle.split(" ").filter((m) => m.length > 2), dans = new Set(t.split(" "));
    return mots.length > 0 && mots.every((m) => dans.has(m)) && cle.split(" ").at(-1) === t.split(" ").at(-1);
  };
  const chicago = async () => {
    const out = [];
    for (const q of noms) for (let page = 1; page <= 5; page++) {
      const d = await sec(`https://api.artic.edu/api/v1/artworks/search?q=${encodeURIComponent(q)}&query[term][is_public_domain]=true&fields=id,title,artist_title,artist_display,date_display,date_end,medium_display,image_id&limit=100&page=${page}`).catch(() => ({ data: [] }));
      for (const o of d.data) {
        if (!o.image_id || !meme(o.artist_title || "")) continue;
        const [ne, mort] = vie(o.artist_display);
        if (!libre(o.date_display, o.date_end, mort, ne)) continue;
        out.push({ id: `chicago-${o.id}`, titre: o.title, date: o.date_display, annee: annees(o.date_display)[0] ?? null, musee: "chicago",
          vignette: `https://www.artic.edu/iiif/2/${o.image_id}/full/400,/0/default.jpg`, lien: `https://www.artic.edu/artworks/${o.id}` });
      }
      if (d.data.length < 100) break;
    }
    return out;
  };
  const cleveland = async () => {
    const out = [];
    for (const q of noms) {
      const d = await sec(`https://openaccess-api.clevelandart.org/api/artworks/?artists=${encodeURIComponent(q)}&cc0=1&has_image=1&limit=1000`).catch(() => ({ data: [] }));
      for (const o of d.data) {
        const c = o.creators?.[0] || {}, artiste = String(c.description || "").split(" (")[0];
        const web = o.images?.web?.url;
        if (!web || !meme(artiste) || !libre(o.creation_date, o.creation_date_latest, Number(c.death_year) || null, Number(c.birth_year) || null)) continue;
        out.push({ id: `cleveland-${o.id}`, titre: o.title, date: o.creation_date, annee: o.creation_date_earliest ?? null, musee: "cleveland", vignette: web, lien: o.url });
      }
    }
    return out;
  };
  const met = async () => {
    const ids = new Set();
    for (const q of noms) {
      const d = await sec(`https://collectionapi.metmuseum.org/public/collection/v1.1/search?artistOrCulture=true&hasImages=true&q=${encodeURIComponent(q)}`).catch(() => ({}));
      for (const i of d.objectIDs || []) if (ids.size < MET_MAX) ids.add(i);
    }
    const fiches = await Promise.all([...ids].map((i) => sec(`https://collectionapi.metmuseum.org/public/collection/v1/objects/${i}`).catch(() => null)));
    const an = (v) => { const n = Number(v); return n && n < 9999 ? n : null; };
    return fiches.filter((o) => o && o.isPublicDomain && o.primaryImageSmall && meme(o.artistDisplayName || "")
      && libre(o.objectDate, o.objectEndDate, an(o.artistEndDate), an(o.artistBeginDate)))
      .map((o) => ({ id: `met-${o.objectID}`, titre: o.title, date: o.objectDate, annee: o.objectBeginDate ?? null, musee: "met",
        vignette: o.primaryImageSmall, lien: o.objectURL }));
  };
  const [a, b, c] = await Promise.all([chicago(), cleveland(), met()]);
  const vus = new Set();
  const oeuvres = [...a, ...b, ...c].filter((o) => !vus.has(o.id) && vus.add(o.id)).sort((x, y) => (x.annee ?? 1e9) - (y.annee ?? 1e9));
  return { nom, total: oeuvres.length, oeuvres, source: "musees" };
}
