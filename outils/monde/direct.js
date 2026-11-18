// Toutes les œuvres d'un artiste, en direct des musées : d'abord l'API ARTEON (le serveur
// Paquets, src/api.rs), et si elle ne répond pas, les API des musées elles-mêmes depuis le
// navigateur. Rien n'est stocké sur le site.
//
// Le moteur est générique : les musées sont décrits dans sources.json (le même fichier que
// l'API Paquets) ; un musée de plus = une entrée, sans code. Mêmes règles que src/api.rs :
// domaine public en Europe comme aux États-Unis (artiste mort en 1955 ou avant, œuvre
// d'avant 1931).

import { lien, plier } from "./commun.js";

/** L'adresse du serveur ARTEON (Heroku). */
export const API = "https://arteon-1ed4594ee106.herokuapp.com";

const DERNIERE_ANNEE = 1930, DERNIERE_MORT = 1955;

const annees = (t) => [...String(t ?? "").matchAll(/(?<!\d)(\d{4})(?!\d)/g)].map((m) => Number(m[1])).filter((a) => a >= 1000 && a <= 2100);
const enTexte = (v) => (v == null || typeof v === "object" ? "" : String(v));
const anneeDe = (v) => { const a = typeof v === "number" ? v : annees(enTexte(v))[0]; return a && a < 9999 ? a : null; };
const chemin = (v, c) => c.split(".").reduce((x, k) => (x == null ? null : x[/^\d+$/.test(k) ? Number(k) : k]), v);
const remplir = (modele, valeurs) => String(modele || "").replace(/\{(\w+)\}/g, (_, k) => valeurs[k] ?? "");

function libre(date, fin, mort, ne) {
  if ([...annees(date), ...(fin != null ? [fin] : [])].some((a) => a > DERNIERE_ANNEE)) return false;
  if (mort != null) return mort <= DERNIERE_MORT;
  if (ne != null) return ne <= 1890;
  return true;
}

function vie(t) {
  const a = annees(t), s = enTexte(t);
  if (!a.length) return [null, null];
  if (a.length === 1) return /born|[–-]\s*$/.test(s) ? [a[0], null] : [null, a[0]];
  return [a[0], a[1]];
}

const lireJson = async (url, entetes = {}) => {
  const r = await fetch(url, { headers: entetes });
  if (!r.ok) throw new Error(`${r.status}`);
  return r.json();
};

let sources;
const chargerSources = () => (sources ??= fetch(lien("monde/sources.json")).then((r) => r.json()).then((d) => d.sources));

let indexStatique;
const chargerIndex = () => (indexStatique ??= fetch(lien("api/index.json")).then((r) => (r.ok ? r.json() : null)).catch(() => null));

/** Toutes les œuvres de l'artiste dans les musées : l'API publiée avec le site (calculée
 * chaque semaine par le moteur de l'API Paquets), sinon le serveur Paquets, sinon les musées
 * eux-mêmes, en direct. */
export async function oeuvresEnDirect(nom, cleNom, alias = {}) {
  // 1. L'API statique, publiée avec le site : immédiate, sans serveur.
  try {
    const index = await chargerIndex();
    const fichier = index && (index.noms?.[nom] || index.cles?.[cleNom(nom)]);
    if (fichier) {
      const r = await fetch(lien(`api/${fichier}.json`));
      if (r.ok) return { ...(await r.json()), source: "statique" };
    }
  } catch { /* pas encore publiée pour cet artiste */ }
  // 2. Le serveur Paquets (avec sa mémoire d'un jour) ; 12 secondes au plus.
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
    const virgule = /^([^,]+), ([^,]+)$/.exec(trouve); // « Hammershøi, Vilhelm »
    if (virgule && meme(`${virgule[2]} ${virgule[1]}`)) return true;
    const t = cleNom(trouve), p = plier(trouve);
    if (t === cle) return true;
    if (/after |^follower|imitator/.test(p)) return false;
    const mots = cle.split(" ").filter((m) => m.length > 2), dans = new Set(t.split(" "));
    return mots.length > 0 && mots.every((m) => dans.has(m)) && cle.split(" ").at(-1) === t.split(" ").at(-1);
  };

  async function interroger(source) {
    const parPage = source.par_page ?? 100, pages = source.pages ?? 1, entetes = source.entetes ?? {};
    let fiches = [], ids = [];
    for (const q of noms) for (let page = 1; page <= pages; page++) {
      const url = source.recherche.replace("{q}", encodeURIComponent(q)).replace("{n}", parPage)
        .replace("{page}", page).replace("{offset}", (page - 1) * parPage).replace("{debut}", (page - 1) * parPage + 1);
      let r;
      try { r = await lireJson(url, entetes); } catch { break; }
      if (source.ids) { ids.push(...(chemin(r, source.ids) || [])); break; }
      const lot = (source.liste ? chemin(r, source.liste) : r) || [];
      fiches.push(...lot);
      if (lot.length < parPage) break;
    }
    if (source.detail) {
      // Les fiches une par une, douze à la fois (300 au plus depuis le navigateur ; l'API
      // Paquets, elle, va jusqu'au bout).
      ids = [...new Set(ids.map(String))].slice(0, Math.min(source.max ?? 3000, 300));
      fiches = [];
      for (let i = 0; i < ids.length; i += 12) {
        const lot = await Promise.all(ids.slice(i, i + 12).map((id) => lireJson(source.detail.replace("{id}", id), entetes).catch(() => null)));
        fiches.push(...lot.filter(Boolean));
      }
    }
    const champ = (f, k) => (source.champs[k] ? chemin(f, source.champs[k]) : null);
    const out = [];
    for (const f of fiches) {
      if ((source.exige || []).some((k) => { const v = champ(f, k); return v == null || v === false || enTexte(v) === ""; })) continue;
      if (Object.entries(source.egal || {}).some(([k, attendu]) => enTexte(champ(f, k)) !== enTexte(attendu))) continue;
      let artiste = enTexte(champ(f, "artiste"));
      if (source.artiste_jusqua) artiste = artiste.split(source.artiste_jusqua)[0];
      if (!meme(artiste)) {
        // Les autres noms possibles (Europeana : les libellés de ses entités).
        const autres = (champ(f, "artistes") || []).map((v) => (v && typeof v === "object" ? enTexte(v.def) : enTexte(v)));
        // Seulement quand le créateur est un identifiant (une adresse) : un photographe nommé
        // qui a photographié l'œuvre n'en est pas l'auteur.
        const identifiant = artiste === "" || artiste.startsWith("http");
        const trouve = identifiant && autres.find((a) => meme(a));
        if (!trouve) continue;
        artiste = trouve;
      }
      let ne = anneeDe(champ(f, "ne")), mort = anneeDe(champ(f, "mort"));
      if (ne == null && mort == null) [ne, mort] = vie(champ(f, "vie"));
      const date = enTexte(champ(f, "date"));
      if (!libre(date, anneeDe(champ(f, "fin")), mort, ne)) continue;
      const valeurs = Object.fromEntries(Object.keys(source.champs).map((k) => [k, enTexte(champ(f, k))]));
      out.push({ id: `${source.cle}-${valeurs.id}`, titre: valeurs.titre, artiste, date, annee: anneeDe(champ(f, "annee")),
        technique: valeurs.technique, musee: source.cle, vignette: remplir(source.vignette, valeurs),
        image: remplir(source.grande, valeurs), lien: remplir(source.lien, valeurs), fournisseur: valeurs.fournisseur || null });
    }
    return out;
  }

  const lots = await Promise.all((await chargerSources()).map((s) => interroger(s).catch(() => [])));
  const vus = new Set();
  const oeuvres = lots.flat().filter((o) => !vus.has(o.id) && vus.add(o.id)).sort((x, y) => (x.annee ?? 1e9) - (y.annee ?? 1e9));
  return { nom, total: oeuvres.length, oeuvres, source: "musees" };
}

/** Les noms des musées décrits (pour les légendes). */
export async function nomsDesMusees() {
  return Object.fromEntries((await chargerSources()).map((s) => [s.cle, s.nom]));
}
