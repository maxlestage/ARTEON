// Ce que partagent les pages Musées, Explorer et Collections du site : l'adresse du site,
// le catalogue, le texte sans accents, les dates des cartels et les siècles (comme dans
// l'application iOS, Library.swift).

/** Le début des adresses du site (`/ARTEON` sur GitHub Pages). */
export const base = location.pathname.replace(/\/(musees|explorer|monde)\/.*$/, "");

export const echapper = (t) => String(t ?? "").replace(/[&<>"']/g, (c) =>
  ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

export const plier = (t) => String(t ?? "").replace(/œ/g, "oe").replace(/Œ/g, "oe").replace(/æ/g, "ae")
  .replace(/Æ/g, "ae").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

export const nombre = (n) => n.toLocaleString("fr-FR");

/** L'adresse d'un chemin du site (`tableaux/van-gogh/iris.jpg`). */
export const lien = (chemin) => `${base}/${String(chemin).replace(/^\//, "")}`;

/** Le catalogue du site : expositions, salles (avec leur plan), artistes, œuvres. */
let catalogue;
export function chargerCatalogue() {
  catalogue ??= fetch(lien("catalogue.json")).then((r) => r.json());
  return catalogue;
}

/** Les adresses des pages du site pour une œuvre, une exposition, une galerie. */
export const pageOeuvre = (ex, w) => lien(`${ex.slug}/oeuvres/${w.slug}/`);
export const pageExposition = (ex) => lien(`${ex.slug}/`);
export const pageGalerie = (ex, w) => lien(`${ex.gallery}/`) + (w ? `#${w.slug}` : "");

const ROMAINS = { I: 1, V: 5, X: 10, L: 50, C: 100 };

/** Le premier siècle en chiffres romains (« XIIᵉ », « IXe », « Iᵉʳ ») et sa place. */
function siecleRomain(t) {
  const c = [...t];
  for (let i = 0; i < c.length; i++) {
    if (!(c[i] in ROMAINS) || (i > 0 && /\p{L}/u.test(c[i - 1]))) continue;
    let j = i;
    while (j < c.length && c[j] in ROMAINS) j++;
    let total = 0, avant = 0;
    for (const x of c.slice(i, j).reverse()) { const v = ROMAINS[x]; total += v < avant ? -v : v; avant = Math.max(avant, v); }
    let k = j;
    if (c[k] === "ᵉ" || c[k] === "e") {
      k++;
      if (c[k] === "ʳ" || c[k] === "r") k = total === 1 ? k + 1 : c.length + 1;
      const fin = k === c.length || (k < c.length && !/\p{L}/u.test(c[k]));
      if (fin && total > 0 && total <= 30) return { valeur: total, place: i };
    }
    i = j - 1;
  }
  return null;
}

/** L'année d'une date de cartel (« 1888 », « vers 2090 av. J.-C. », « XIIᵉ siècle »). */
export function annee(texte) {
  const t = String(texte ?? "");
  const bas = t.toLowerCase();
  if (bas.includes("milliard") || bas.includes("million")) return -1000000;
  const avantJC = /av\. j\.?-c|avant j/.test(bas);
  const m = /\d+/.exec(t);
  const s = siecleRomain(t);
  if (s && (!m || s.place < m.index)) { const y = (s.valeur - 1) * 100 + 50; return avantJC ? -y : y; }
  if (!m) return null;
  return avantJC ? -Number(m[0]) : Number(m[0]);
}

export function romain(n) {
  const t = [[1000, "M"], [900, "CM"], [500, "D"], [400, "CD"], [100, "C"], [90, "XC"], [50, "L"], [40, "XL"], [10, "X"], [9, "IX"], [5, "V"], [4, "IV"], [1, "I"]];
  let s = "";
  for (const [v, l] of t) while (n >= v) { s += l; n -= v; }
  return s;
}

export const rangSiecle = (y) => y <= -100000 ? -999999 : (y < 0 ? -(Math.floor((-y - 1) / 100) + 1) : Math.floor((y - 1) / 100) + 1);

export function titreSiecle(r) {
  if (r === -999999) return "Avant l'histoire";
  const n = Math.abs(r);
  return `${romain(n)}${n === 1 ? "ᵉʳ" : "ᵉ"} siècle${r < 0 ? " av. J.-C." : ""}`;
}

/** Les adresses en #/… : { type, valeur }. */
export function route() {
  const [, type, ...reste] = decodeURIComponent(location.hash.replace(/^#\/?/, "#/")).split("/");
  return { type: type || "", valeur: reste.join("/") };
}

/** La vignette d'une œuvre du site, et sa carte. */
export function carteOeuvre(ex, w, sousTitre) {
  const ligne = sousTitre ?? [w.artist, w.year].filter(Boolean).join(" · ");
  return `<a class="monde-carte" href="${pageOeuvre(ex, w)}"><img loading="lazy" decoding="async" src="${lien(w.image)}" alt=""><b>${echapper(w.title)}</b><span>${echapper(ligne)}</span></a>`;
}

/** Toutes les œuvres du catalogue, avec leur exposition. */
export const toutes = (c) => c.exhibitions.flatMap((ex) => ex.works.map((w) => ({ ex, w })));
