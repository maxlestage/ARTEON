// Ce que partagent les pages Musées, Explorer et Collections du site : l'adresse du site,
// le catalogue, le texte sans accents, les dates des cartels et les siècles (comme dans
// l'application iOS, Library.swift).

import { jsonGarde } from "./memoire.js";

/** La langue de la page : celle de `<html lang>` (fr, en ou es). Les pages en anglais et en
 * espagnol sont sous /en/ et /es/ ; les fichiers (images, données) sont communs. */
export const langue = ["en", "es"].includes(document.documentElement.lang) ? document.documentElement.lang : "fr";
const prefixe = langue === "fr" ? "" : `/${langue}`;

/** Un texte de l'interface dans la langue de la page : `t("Œuvres", "Works", "Obras")`. */
export const t = (fr, en, es) => (langue === "en" ? en : langue === "es" ? es : fr);

/** Le début des adresses du site (`/ARTEON` sur GitHub Pages), sans la langue. */
// D'après l'adresse de ce fichier (`<base>/monde/commun.js`) : juste sur toutes les pages.
export const base = new URL("..", import.meta.url).pathname.replace(/\/$/, "");

export const echapper = (t) => String(t ?? "").replace(/[&<>"']/g, (c) =>
  ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

export const plier = (t) => String(t ?? "").replace(/œ/g, "oe").replace(/Œ/g, "oe").replace(/æ/g, "ae")
  .replace(/Æ/g, "ae").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

export const nombre = (n) => n.toLocaleString(t("fr-FR", "en-GB", "es-ES"));

/** L'adresse d'un chemin du site (`tableaux/van-gogh/iris.jpg`). */
export const lien = (chemin) => `${base}/${String(chemin).replace(/^\//, "")}`;

/** L'adresse d'une page du site dans la langue de la page (`credits/` → `/ARTEON/en/credits/`). */
export const lienPage = (chemin) => `${base}${prefixe}/${String(chemin).replace(/^\//, "")}`;

/** Le catalogue du site : expositions, salles (avec leur plan), artistes, œuvres. */
let catalogue;
export function chargerCatalogue() {
  catalogue ??= jsonGarde(lien(langue === "fr" ? "catalogue.json" : `${langue}/catalogue.json`), 3600e3);
  return catalogue;
}

/** Les adresses des pages du site pour une œuvre, une exposition, une galerie. */
export const pageOeuvre = (ex, w) => lienPage(`${ex.slug}/oeuvres/${w.slug}/`);
export const pageExposition = (ex) => lienPage(`${ex.slug}/`);
export const pageGalerie = (ex, w) => lienPage(`${ex.gallery}/`) + (w ? `#${w.slug}` : "");

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
  if (r === -999999) return t("Avant l'histoire", "Prehistory", "Prehistoria");
  const n = Math.abs(r);
  if (langue === "en") {
    const suffixe = n % 100 >= 11 && n % 100 <= 13 ? "th" : ({ 1: "st", 2: "nd", 3: "rd" })[n % 10] || "th";
    return `${n}${suffixe} century${r < 0 ? " BC" : ""}`;
  }
  if (langue === "es") return `Siglo ${romain(n)}${r < 0 ? " a. C." : ""}`;
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

/** Affiche TOUS les `elements` dans `conteneur` (une carte par élément), sans bouton et sans
 * attendre que l'on descende : par paquets, pour que la page reste fluide pendant qu'elle se
 * remplit. `progres(montres, total)` est appelé après chaque paquet. Au-delà de `max`
 * éléments (des dizaines de milliers : la mémoire d'un téléphone n'y suffirait pas), la suite
 * arrive quand la fin de la liste approche de l'écran. */
export function toutAfficher(conteneur, elements, carte, progres, { lot = 400, max = 12000 } = {}) {
  let montres = 0;
  const fin = document.createElement("div");
  conteneur.after(fin);
  const proche = () => fin.getBoundingClientRect().top < innerHeight + 2500;
  const paquet = () => {
    if (!conteneur.isConnected || montres >= elements.length) return;
    conteneur.insertAdjacentHTML("beforeend", elements.slice(montres, montres + lot).map(carte).join(""));
    montres = Math.min(montres + lot, elements.length);
    progres?.(montres, elements.length);
    if (montres >= elements.length) { fin.remove(); return; }
    // Tout de suite jusqu'à `max` ; ensuite seulement si la fin est près de l'écran.
    if (montres < max || proche()) setTimeout(paquet, 40);
  };
  paquet();
  // Au-delà de `max` : la suite quand on arrive en bas (vérifié aussi après chaque défilement).
  const reprendre = () => {
    if (!conteneur.isConnected || montres >= elements.length) { removeEventListener("scroll", reprendre); return; }
    if (montres >= max && proche()) paquet();
  };
  addEventListener("scroll", reprendre, { passive: true });
  new IntersectionObserver((e) => { if (e.some((x) => x.isIntersecting)) reprendre(); }, { rootMargin: "2500px" }).observe(fin);
}

/** Une rubrique repliable (comme sur les pages des œuvres) : un titre en capitales, un « + »,
 * et son contenu (`html`, déjà échappé) qui s'ouvre d'un toucher. */
export const rubrique = (titre, html, ouverte = false) =>
  `<details class="fold"${ouverte ? " open" : ""}><summary><span>${echapper(titre)}</span><span class="fold-sign" aria-hidden="true"></span></summary><div class="fold-body">${html}</div></details>`;

/** Un renseignement du cartel : son nom, sa valeur dessous. */
export const renseignement = (nom, valeur) => (valeur ? `<div class="fact"><dt>${echapper(nom)}</dt><dd>${echapper(valeur)}</dd></div>` : "");

/** Le choix de la vue des grilles d'œuvres : « Mosaïque » (des vignettes serrées, pour tout
 *  voir d'un coup d'œil et toucher celle qu'on veut ; par défaut) ou « Avec les titres » (les
 *  cartes). Retenu pour la suite (dans ce navigateur). Renvoie les deux boutons, à placer. */
export function choixVue() {
  let vue = "mosaique";
  try { vue = localStorage.getItem("arteon-vue") || vue; } catch { /* navigation privée */ }
  const choix = document.createElement("div");
  choix.className = "monde-vue";
  choix.setAttribute("role", "group");
  choix.setAttribute("aria-label", t("Affichage des œuvres", "How works are shown", "Visualización de las obras"));
  choix.innerHTML = `<button type="button" data-vue="mosaique">▦ ${t("Mosaïque", "Mosaic", "Mosaico")}</button>`
    + `<button type="button" data-vue="cartes">☰ ${t("Avec les titres", "With titles", "Con los títulos")}</button>`;
  const appliquer = () => {
    document.documentElement.classList.toggle("monde-mosaique", vue === "mosaique");
    for (const b of document.querySelectorAll(".monde-vue button")) b.setAttribute("aria-pressed", String(b.dataset.vue === vue));
  };
  choix.addEventListener("click", (e) => {
    const b = e.target.closest("button[data-vue]");
    if (!b) return;
    vue = b.dataset.vue;
    try { localStorage.setItem("arteon-vue", vue); } catch { /* navigation privée */ }
    appliquer();
  });
  for (const b of choix.querySelectorAll("button")) b.setAttribute("aria-pressed", String(b.dataset.vue === vue));
  document.documentElement.classList.toggle("monde-mosaique", vue === "mosaique");
  return choix;
}

/** Les images de Wikimedia Commons : seules les tailles courantes sont toujours prêtes ; les
 *  autres sont fabriquées à la demande, et Commons les refuse (429) quand on en demande
 *  beaucoup d'un coup (une galerie de 200 œuvres). Une image refusée est redemandée en
 *  250 px (toujours prête), puis en 500, puis encore en 250 un peu plus tard ; sinon la
 *  carte montre son titre à la place. Installé une fois, pour toutes les pages. */
function reparerImages() {
  document.addEventListener("error", (e) => {
    const img = e.target;
    if (!(img instanceof HTMLImageElement)) return;
    // Une image refusée ailleurs (le pare-feu d'un musée, une image retirée) : la carte montre
    // son titre plutôt qu'une image cassée.
    if (!/wikimedia\.org\//.test(img.src)) { img.closest(".monde-carte, .monde-sujet")?.classList.add("sans-image"); return; }
    const essais = Number(img.dataset.essais || 0);
    if (essais >= 3) { img.closest(".monde-carte, .monde-sujet")?.classList.add("sans-image"); return; }
    img.dataset.essais = String(essais + 1);
    const u = new URL(img.src);
    u.searchParams.set("width", ["250", "500", "250"][essais]);
    setTimeout(() => { img.src = u.toString(); }, 1200 * essais);
  }, true);
}
if (typeof document !== "undefined") reparerImages();
