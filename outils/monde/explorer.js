// Explorer le musée autrement, sur le site : les artistes de A à Z (et leurs œuvres dans
// les collections du monde), la chronologie siècle par siècle, les thèmes, et le jeu
// « Qui l'a fait ? » (le record est gardé dans le navigateur).

import { annee, carteOeuvre, chargerCatalogue, echapper, lien, lienPage, nombre, plier, rangSiecle, route, t, titreSiecle, toutes } from "./commun.js";
import { nomsDesMusees, oeuvresEnDirect } from "./direct.js";
import { jsonGarde } from "./memoire.js";

const racine = document.getElementById("monde");
let C, oeuvres, artistes, siecles, themes;

function indexer() {
  oeuvres = toutes(C);
  const parNom = new Map();
  for (const ex of C.exhibitions) {
    for (const a of ex.artists) {
      const e = parNom.get(a.name) || { nom: a.name, vie: "", style: "", bio: "", oeuvres: [] };
      if (a.biography.length > e.bio.length) Object.assign(e, { vie: a.life || e.vie, style: a.style || e.style, bio: a.biography });
      parNom.set(a.name, e);
    }
    for (const w of ex.works) {
      if (!w.artist) continue;
      const e = parNom.get(w.artist) || { nom: w.artist, vie: "", style: "", bio: "", oeuvres: [] };
      e.oeuvres.push({ ex, w });
      parNom.set(w.artist, e);
    }
  }
  artistes = [...parNom.values()].filter((a) => a.oeuvres.length).sort((a, b) => plier(a.nom).localeCompare(plier(b.nom)));
  const parSiecle = new Map();
  for (const o of oeuvres) {
    const y = annee(o.w.year);
    const r = y == null ? null : rangSiecle(y);
    if (!parSiecle.has(r)) parSiecle.set(r, []);
    parSiecle.get(r).push({ ...o, y });
  }
  siecles = [...parSiecle].filter(([r]) => r != null).sort((a, b) => a[0] - b[0])
    .map(([r, os]) => ({ r, titre: titreSiecle(r), oeuvres: os.sort((a, b) => a.y - b.y) }));
  const parTheme = new Map();
  for (const o of oeuvres) for (const t of o.w.notice?.themes || []) {
    const k = plier(t);
    if (!parTheme.has(k)) parTheme.set(k, { titre: t, oeuvres: [] });
    parTheme.get(k).oeuvres.push(o);
  }
  themes = [...parTheme.values()].sort((a, b) => b.oeuvres.length - a.oeuvres.length);
}

const oeuvresN = (n) => (n === 1 ? t("1 œuvre", "1 work", "1 obra") : `${nombre(n)} ${t("œuvres", "works", "obras")}`);

const menu = (actif) => `
  <h1 class="monde-titre">${t("Explorer", "Explore", "Explorar")}</h1>
  <div class="monde-onglets">${[["", t("Les artistes", "Artists", "Los artistas")], ["chronologie", t("La chronologie", "Timeline", "La cronología")],
    ["themes", t("Les thèmes", "Themes", "Los temas")], ["jeu", t("Qui l'a fait ?", "Who made it?", "¿Quién la hizo?")]]
    .map(([k, nom]) => `<a class="${actif === k ? "actif" : ""}" href="#/${k}">${nom}</a>`).join("")}</div>`;

// Le catalogue de tous les artistes : ceux des expositions (avec leur biographie) et ceux
// des collections du monde, réunis par nom. Chargé à la première visite des artistes.
let catalogueArtistes;

// Un même artiste porte des noms différents selon les musées : « Rembrandt (Rembrandt van
// Rijn) », « Rembrandt van Rijn », « Rembrandt » ; « Titian » et « Titien »… La clé réunit
// ces formes : sans accents ni ponctuation, la forme longue entre parenthèses, et quelques
// noms français et anglais connus.
// Les variantes connues (« titian » → « titien ») : outils/monde/alias.json, le même fichier
// pour le site, son pied de page et l'application.
let ALIAS = {};
function cleNom(nom) {
  let n = String(nom ?? "").trim();
  const m = /^(.+?)\s*\((.+)\)$/.exec(n);
  if (m) {
    const premier = plier(m[1]).split(/\s+/)[0];
    n = plier(m[2]).includes(premier) ? m[2] : m[1];
  }
  const k = plier(n).replace(/[-‐–'’.,]/g, " ").replace(/\s+/g, " ").trim();
  return ALIAS[k] ?? k;
}

function chargerArtistes() {
  catalogueArtistes ??= Promise.all([
    jsonGarde(lien("monde/alias.json")).catch(() => ({})).then((a) => { ALIAS = a; }),
    jsonGarde(lien("monde/donnees/liste.json")).catch(() => []),
    jsonGarde(lien("monde/donnees/index.json")).catch(() => ({ ids: [] })),
    // L'annuaire : tous les artistes du domaine public dont un musée montre une œuvre
    // (outils/annuaire.py) : [nom, naissance, mort, œuvres].
    jsonGarde(lien("monde/annuaire.json"), 7 * 24 * 3600e3).catch(() => []),
  ]).then(([, liste, index, annuaire]) => {
    const parCle = new Map();
    for (const a of artistes) {
      const cle = cleNom(a.nom), deja = parCle.get(cle);
      if (deja) { deja.oeuvres.push(...a.oeuvres); if (a.bio.length > deja.bio.length) Object.assign(deja, { vie: a.vie, style: a.style, bio: a.bio }); }
      else parCle.set(cle, { ...a, oeuvres: [...a.oeuvres], monde: [] });
    }
    liste.forEach((l, i) => {
      const nom = l[1];
      if (!nom || nom === "Artiste inconnu") return;
      const cle = cleNom(nom);
      if (!parCle.has(cle)) parCle.set(cle, { nom, vie: "", style: "", bio: "", oeuvres: [], monde: [] });
      parCle.get(cle).monde.push({ l, id: index.ids[i] });
    });
    for (const [nom, ne, mort, n] of annuaire) {
      const cle = cleNom(nom), deja = parCle.get(cle);
      const vie = ne && mort ? `${ne}–${mort}` : mort ? `${t("mort en", "died", "murió en")} ${mort}` : ne ? `${t("né en", "born", "nació en")} ${ne}` : "";
      if (deja) { deja.ailleurs = Math.max(deja.ailleurs || 0, n); deja.vie ||= vie; }
      else parCle.set(cle, { nom, vie, style: "", bio: "", oeuvres: [], monde: [], ailleurs: n });
    }
    const tous = [...parCle.values()].sort((a, b) => plier(a.nom).localeCompare(plier(b.nom)));
    return { tous, parCle };
  });
  return catalogueArtistes;
}

// Le nombre d'œuvres connues : celles d'ARTEON, ou celles que les musées montrent ailleurs.
const total = (a) => Math.max(a.oeuvres.length + a.monde.length, a.ailleurs || 0);
const lettreDe = (nom) => { const l = (plier(nom).replace(/^[^a-z0-9]+/, "")[0] || "#").toUpperCase(); return /[A-Z]/.test(l) ? l : "#"; };
const vignetteMonde = ({ l, id }) => l[7] || lien(`monde/v/${id}.webp`);
const vignetteDe = (a) => (a.oeuvres[0] ? lien(a.oeuvres[0].w.image) : a.monde[0] ? vignetteMonde(a.monde[0]) : "");

async function listeArtistes(choisie) {
  racine.innerHTML = menu("") + `<p class="monde-chargement">${t("Ouverture du catalogue des artistes…", "Opening the catalogue of artists…", "Abriendo el catálogo de artistas…")}</p>`;
  const { tous } = await chargerArtistes();
  if (route().type !== "" && route().type !== "lettre") return;
  const lettres = [...new Set(tous.map((a) => lettreDe(a.nom)))].sort((a, b) => (a === "#") - (b === "#") || a.localeCompare(b));
  const lettre = lettres.includes(choisie) ? choisie : lettres[0];
  racine.innerHTML = menu("") + `
    <p class="monde-lead">${t(
      `${nombre(tous.length)} artistes, ateliers et cultures : ceux des expositions, ceux des
      collections du monde, et tous ceux dont un musée montre une œuvre libre de droits. Pour chacun, le
      catalogue complet de ses œuvres. Un artiste absent de la liste ? Écrire son nom : ARTEON le cherche
      dans les musées.`,
      `${nombre(tous.length)} artists, workshops and cultures: those in the exhibitions, those in the
      world collections, and every one whose copyright-free work is shown by a museum. For each, the
      complete catalogue of their works. An artist missing from the list? Type their name: ARTEON will search
      the museums for them.`,
      `${nombre(tous.length)} artistas, talleres y culturas: los de las exposiciones, los de las
      colecciones del mundo y todos aquellos de los que un museo muestra una obra libre de derechos. Para cada uno,
      el catálogo completo de sus obras. ¿Falta un artista en la lista? Escriba su nombre: ARTEON lo busca
      en los museos.`)}</p>
    <div class="monde-barre"><input type="search" id="filtre" placeholder="${t("Un nom, un style…", "A name, a style…", "Un nombre, un estilo…")}" aria-label="${t("Chercher un artiste", "Search for an artist", "Buscar un artista")}"></div>
    <nav class="monde-alphabet" aria-label="${t("Les lettres", "Letters", "Las letras")}">${lettres.map((l) => `<a class="${l === lettre ? "actif" : ""}" href="#/lettre/${encodeURIComponent(l)}">${l}</a>`).join("")}</nav>
    <div id="artistes"></div>`;
  const zone = document.getElementById("artistes");
  const ligne = (a) => `<a class="monde-artiste-ligne" href="#/artiste/${encodeURIComponent(a.nom)}">${vignetteDe(a) ? `<img loading="lazy" src="${vignetteDe(a)}" alt="">` : "<i></i>"}
      <span><b>${echapper(a.nom)}</b><small>${echapper([a.vie, a.style].filter(Boolean).join(" · ") || (a.oeuvres.length ? t("Dans les expositions", "In the exhibitions", "En las exposiciones")
        : a.monde.length ? t("Dans les collections du monde", "In the world collections", "En las colecciones del mundo")
        : t("Dans les musées du monde", "In the world's museums", "En los museos del mundo")))}</small></span><em>${nombre(total(a))}</em></a>`;
  const dessiner = (q) => {
    const f = plier(q.trim());
    if (f.length >= 2) {
      // D'abord les noms qui contiennent la recherche, puis les styles et les vies ; les
      // plus riches en œuvres en premier.
      const parNom = tous.filter((a) => plier(a.nom).includes(f)).sort((x, y) => total(y) - total(x));
      const vus = new Set(parNom);
      const trouves = parNom.concat(tous.filter((a) => !vus.has(a) && plier(`${a.style} ${a.vie}`).includes(f)));
      // N'importe quel artiste : le nom écrit se cherche tel quel dans tous les musées.
      const ecrit = q.trim();
      const ailleurs = f.length >= 3 && !trouves.some((a) => plier(a.nom) === f)
        ? `<a class="monde-artiste-ligne" href="#/artiste/${encodeURIComponent(ecrit)}"><i></i><span><b>${t(`Chercher « ${echapper(ecrit)} » dans tous les musées`, `Search all museums for “${echapper(ecrit)}”`, `Buscar «${echapper(ecrit)}» en todos los museos`)}</b><small>${t("Son catalogue, en direct des musées du monde", "Their catalogue, live from the world's museums", "Su catálogo, en directo de los museos del mundo")}</small></span><em>→</em></a>` : "";
      zone.innerHTML = (trouves.length
        ? `<p class="monde-compte">${nombre(trouves.length)} ${trouves.length > 1 ? t("artistes", "artists", "artistas") : t("artiste", "artist", "artista")}${trouves.length > 300 ? ` ${t("(les 300 premiers)", "(the first 300)", "(los 300 primeros)")}` : ""}</p>`
        : `<p class="monde-chargement">${t("Pas encore dans la liste.", "Not in the list yet.", "Todavía no está en la lista.")}</p>`)
        + `<div class="monde-artistes">${trouves.slice(0, 300).map(ligne).join("")}${ailleurs}</div>`;
      return;
    }
    // Une lettre compte des milliers d'artistes : 300 à la fois.
    const ici = tous.filter((a) => lettreDe(a.nom) === lettre);
    zone.innerHTML = `<h2 class="monde-lettre">${lettre} <small class="monde-note">${nombre(ici.length)} ${ici.length > 1 ? t("artistes", "artists", "artistas") : t("artiste", "artist", "artista")}</small></h2><div class="monde-artistes" id="lettre-liste"></div>
      <button class="monde-plus" id="lettre-plus" type="button">${t("Afficher plus", "Show more", "Mostrar más")}</button>`;
    const liste = document.getElementById("lettre-liste"), bouton = document.getElementById("lettre-plus");
    let montres = 0;
    const plus = () => {
      liste.insertAdjacentHTML("beforeend", ici.slice(montres, montres + 300).map(ligne).join(""));
      montres += 300;
      bouton.hidden = montres >= ici.length;
    };
    bouton.addEventListener("click", plus);
    plus();
    // La suite s'affiche d'elle-même en descendant.
    new IntersectionObserver((e) => { if (e[0].isIntersecting) plus(); }, { rootMargin: "1200px" }).observe(bouton);
  };
  document.getElementById("filtre").addEventListener("input", (e) => dessiner(e.target.value));
  dessiner("");
}

/** Une carte des collections du monde : la vignette, le titre, la date et le musée. */
const carteMonde = ({ l, id }) => `<a class="monde-carte" href="${lienPage(`monde/#/oeuvre/${id}`)}"><img loading="lazy" decoding="async" src="${vignetteMonde({ l, id })}" alt=""><b>${echapper(l[0])}</b><span>${echapper([l[2], t("collections du monde", "world collections", "colecciones del mundo")].filter(Boolean).join(" · "))}</span></a>`;

async function artiste(nom) {
  racine.innerHTML = `<p class="monde-chargement">${t("Ouverture du catalogue…", "Opening the catalogue…", "Abriendo el catálogo…")}</p>`;
  const { parCle } = await chargerArtistes();
  // Un artiste qu'ARTEON ne montre pas encore : son catalogue vient tout entier des musées.
  const a = parCle.get(cleNom(nom)) || { nom, vie: "", style: "", bio: "", oeuvres: [], monde: [] };
  // Le catalogue complet, du plus ancien au plus récent : les œuvres des expositions et
  // celles des collections du monde, ensemble ; puis, en direct, celles des musées.
  const toutes = [
    ...a.oeuvres.map(({ ex, w }) => ({ y: annee(w.year), html: carteOeuvre(ex, w, [w.year, ex.short_name || ex.title].filter(Boolean).join(" · ")) })),
    ...a.monde.map((m) => ({ y: m.l[3], html: carteMonde(m) })),
  ].sort((x, y) => (x.y ?? 1e9) - (y.y ?? 1e9));
  const titre = (n) => `${t("Le catalogue complet : ", "Complete catalogue: ", "El catálogo completo: ")}${oeuvresN(n)}`;
  const enDirect = t("Dans les musées, en direct", "Live from the museums", "En los museos, en directo");
  racine.innerHTML = `<a class="monde-retour" href="#/lettre/${encodeURIComponent(lettreDe(a.nom))}">← ${t("Les artistes", "Artists", "Los artistas")}</a>
    <h1 class="monde-titre">${echapper(a.nom)}</h1>
    <p class="monde-artiste">${echapper([a.vie, a.style].filter(Boolean).join(" · "))}</p>
    ${a.bio ? `<p class="monde-texte">${echapper(a.bio)}</p>` : ""}
    <section class="monde-section"><h2 id="catalogue-titre">${titre(toutes.length)}</h2>
      <p class="monde-note">${[
        a.oeuvres.length ? `${nombre(a.oeuvres.length)} ${t("dans les expositions du musée", "in the museum's exhibitions", "en las exposiciones del museo")}` : "",
        a.monde.length ? `${nombre(a.monde.length)} ${t("dans les collections du monde", "in the world collections", "en las colecciones del mundo")}` : "",
        toutes.length ? t("de la plus ancienne à la plus récente", "from oldest to most recent", "de la más antigua a la más reciente")
          : t("Ses œuvres viennent des musées, en direct", "Their works come live from the museums", "Sus obras llegan de los museos, en directo"),
      ].filter(Boolean).join(" · ")}</p>
      <div class="monde-grille">${toutes.map((o) => o.html).join("")}</div></section>
    <section class="monde-section" id="en-direct"><h2>${enDirect}</h2>
      <p class="monde-chargement">${t("Recherche de toutes ses œuvres libres de droits dans les musées du monde…", "Searching the world's museums for all their copyright-free works…", "Buscando todas sus obras libres de derechos en los museos del mundo…")}</p></section>`;
  window.scrollTo(0, 0);
  const zone = document.getElementById("en-direct");
  try {
    const deja = new Set(a.monde.map((m) => m.id));
    const r = await oeuvresEnDirect(a.nom, cleNom, ALIAS);
    if (route().valeur !== nom) return;
    const nouvelles = r.oeuvres.filter((o) => !deja.has(o.id));
    const musee = await nomsDesMusees().catch(() => ({}));
    document.getElementById("catalogue-titre").textContent = titre(toutes.length + nouvelles.length);
    const n = nouvelles.length;
    zone.innerHTML = n ? `<h2>${enDirect}${t(" : ", ": ", ": ")}${toutes.length
        ? (n > 1 ? `${nombre(n)} ${t("œuvres de plus", "more works", "obras más")}` : t("1 œuvre de plus", "1 more work", "1 obra más"))
        : oeuvresN(n)}</h2>
      <p class="monde-note">${t("Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931) · images et fiches chez les musées",
        "Public domain (artist died in 1955 or earlier, work made before 1931) · images and records hosted by the museums",
        "Dominio público (artista fallecido en 1955 o antes, obra anterior a 1931) · imágenes y fichas en los museos")}${r.source === "musees"
        ? ` · ${t("directement chez les musées", "directly from the museums", "directamente de los museos")}` : ` · ${t("par l'API ARTEON", "via the ARTEON API", "a través de la API de ARTEON")}`}</p>
      <div class="monde-grille" id="direct-grille"></div><button class="monde-plus" id="direct-plus" type="button">${t("Afficher plus", "Show more", "Mostrar más")}</button>`
      : `<h2>${enDirect}</h2><p class="monde-note">${t("Aucune autre œuvre libre de droits dans les collections ouvertes des musées.", "No other copyright-free works in the museums' open collections.", "Ninguna otra obra libre de derechos en las colecciones abiertas de los museos.")}</p>`;
    // 300 à la fois : les catalogues des grands artistes comptent des milliers d'œuvres.
    const grille = document.getElementById("direct-grille"), bouton = document.getElementById("direct-plus");
    let montrees = 0;
    const carte = (o) => `<a class="monde-carte" href="${echapper(o.lien)}" target="_blank" rel="noopener"><img loading="lazy" decoding="async" src="${echapper(o.vignette)}" alt=""><b>${echapper(o.titre)}</b><span>${echapper([o.date, o.fournisseur && !o.fournisseur.startsWith("http") ? `${o.fournisseur} · ${(musee[o.musee] || "").split(" (")[0]}` : musee[o.musee]].filter(Boolean).join(" · "))}</span></a>`;
    const plus = () => {
      if (!grille) return;
      grille.insertAdjacentHTML("beforeend", nouvelles.slice(montrees, montrees + 300).map(carte).join(""));
      montrees += 300;
      if (bouton) bouton.hidden = montrees >= nouvelles.length;
    };
    bouton?.addEventListener("click", plus);
    plus();
    if (bouton) new IntersectionObserver((e) => { if (e[0].isIntersecting) plus(); }, { rootMargin: "1200px" }).observe(bouton);
  } catch {
    zone.innerHTML = `<h2>${enDirect}</h2><p class="monde-note">${t("Les musées n'ont pas répondu. Réessayer plus tard.", "The museums did not respond. Please try again later.", "Los museos no han respondido. Vuelva a intentarlo más tarde.")}</p>`;
  }
}

function chronologie() {
  racine.innerHTML = menu("chronologie") + `
    <p class="monde-lead">${t("Toutes les œuvres du musée, de la plus ancienne à la plus récente, siècle par siècle.", "Every work in the museum, from oldest to most recent, century by century.", "Todas las obras del museo, de la más antigua a la más reciente, siglo a siglo.")}</p>
    <div class="monde-barre"><select id="aller" aria-label="${t("Aller au siècle", "Go to century", "Ir al siglo")}"><option value="">${t("Aller au siècle…", "Go to century…", "Ir al siglo…")}</option>
      ${siecles.map((s) => `<option value="s${s.r}">${echapper(s.titre)} (${nombre(s.oeuvres.length)})</option>`).join("")}</select></div>
    ${siecles.map((s) => `<section class="monde-section" id="s${s.r}"><h2>${echapper(s.titre)} <small class="monde-note">${oeuvresN(s.oeuvres.length)} ·
        <a href="${lienPage(`monde/#/siecle/${s.r}`)}">${t("dans les collections du monde", "in the world collections", "en las colecciones del mundo")}</a></small></h2>
      <div class="monde-rangee">${s.oeuvres.map(({ ex, w }) => carteOeuvre(ex, w)).join("")}</div></section>`).join("")}`;
  document.getElementById("aller").addEventListener("change", (e) => document.getElementById(e.target.value)?.scrollIntoView({ behavior: "smooth" }));
}

function listeThemes(choisi) {
  const theme = choisi && themes.find((x) => x.titre === choisi);
  racine.innerHTML = menu("themes") + (theme ? `
    <a class="monde-retour" href="#/themes">← ${t("Tous les thèmes", "All themes", "Todos los temas")}</a><h2>${echapper(theme.titre)} <small class="monde-note">${oeuvresN(theme.oeuvres.length)}</small></h2>
    <div class="monde-grille">${theme.oeuvres.map(({ ex, w }) => carteOeuvre(ex, w)).join("")}</div>` : `
    <p class="monde-lead">${nombre(themes.length)} ${t("thèmes : la nuit, la mer, l'amour, la mort, le portrait…", "themes: night, the sea, love, death, the portrait…", "temas: la noche, el mar, el amor, la muerte, el retrato…")}</p>
    <div class="monde-themes">${themes.map((x) => `<a href="#/themes/${encodeURIComponent(x.titre)}">${echapper(x.titre)} <span>${x.oeuvres.length}</span></a>`).join("")}</div>`);
  window.scrollTo(0, 0);
}

// Le jeu : une œuvre, quatre artistes de la même époque. Une série, et un record.
const nomme = (n) => {
  const f = plier(n);
  return !/anonyme|atelier|epoque|dynastie|empire|royaume|manufacture|fours|tradition|style|moines|scriptorium|faiencerie|maison |mines|gisement|formation|falaises|recifs|forets|coulees|massif|schistes|orfevres|tisserands|couturieres|armuriers|peintres-|enlumineurs|tapissiers|mosaistes|ceramistes|studios/.test(f)
    && !n.includes(",") && n.split(" ").length >= 2;
};
let serie = 0;
const record = () => { try { return Number(localStorage.getItem("arteon-record") || 0); } catch { return 0; } };
const garderRecord = (n) => { try { localStorage.setItem("arteon-record", String(n)); } catch { /* navigation privée */ } };

function jeu() {
  const candidates = oeuvres.filter((o) => nomme(o.w.artist));
  const { ex, w } = candidates[Math.floor(Math.random() * candidates.length)];
  const y = annee(w.year);
  const proches = artistes.filter((a) => nomme(a.nom) && a.nom !== w.artist && (y == null || a.oeuvres.some((o) => { const z = annee(o.w.year); return z != null && Math.abs(z - y) < 120; })));
  const reserve = proches.length >= 3 ? proches : artistes.filter((a) => nomme(a.nom) && a.nom !== w.artist);
  const choix = [w.artist];
  while (choix.length < 4 && reserve.length) {
    const a = reserve.splice(Math.floor(Math.random() * reserve.length), 1)[0].nom;
    if (!choix.includes(a)) choix.push(a);
  }
  choix.sort(() => Math.random() - 0.5);
  racine.innerHTML = menu("jeu") + `
    <p class="monde-lead">🔥 ${t("Série :", "Streak:", "Racha:")} <b>${serie}</b> · 🏆 ${t("Record :", "Best:", "Récord:")} <b>${record()}</b></p>
    <div class="jeu-image"><img src="${lien(w.image)}" alt="${t("L'œuvre à reconnaître", "The work to identify", "La obra que hay que reconocer")}"></div>
    <h2>${t("Qui a fait cette œuvre ?", "Who made this work?", "¿Quién hizo esta obra?")}</h2>
    <div class="jeu-choix">${choix.map((c) => `<button type="button" data-nom="${echapper(c)}">${echapper(c)}</button>`).join("")}</div>
    <div id="jeu-suite"></div>`;
  racine.querySelectorAll(".jeu-choix button").forEach((b) => b.addEventListener("click", () => {
    const bon = b.dataset.nom === w.artist;
    racine.querySelectorAll(".jeu-choix button").forEach((x) => {
      x.disabled = true;
      if (x.dataset.nom === w.artist) x.classList.add("bon");
      else if (x === b) x.classList.add("faux");
    });
    serie = bon ? serie + 1 : 0;
    if (serie > record()) garderRecord(serie);
    document.getElementById("jeu-suite").innerHTML = `
      <p><b>${echapper(w.title)}</b> · ${echapper(w.year)} · ${echapper(ex.short_name || ex.title)}</p>
      <div class="monde-nav"><a href="${lienPage(`${ex.slug}/oeuvres/${w.slug}/`)}">${t("Voir la notice", "View the description", "Ver la ficha")}</a><a href="#/jeu" id="jeu-encore">${bon ? `${t("Suivante", "Next", "Siguiente")} ›` : t("Rejouer", "Play again", "Volver a jugar")}</a></div>`;
    document.getElementById("jeu-encore").addEventListener("click", (e) => { e.preventDefault(); jeu(); });
  }));
}

function afficher() {
  const r = route();
  if (r.type === "artiste") return artiste(r.valeur);
  if (r.type === "chronologie") return chronologie();
  if (r.type === "themes") return listeThemes(r.valeur);
  if (r.type === "jeu") return jeu();
  if (r.type === "lettre") return listeArtistes(r.valeur);
  listeArtistes();
}

racine.innerHTML = `<p class="monde-chargement">${t("Ouverture du musée…", "Opening the museum…", "Abriendo el museo…")}</p>`;
chargerCatalogue().then((c) => {
  C = c;
  indexer();
  window.addEventListener("hashchange", afficher);
  afficher();
}).catch((e) => { racine.innerHTML = `<p class="monde-chargement">${t("Le musée n'a pas pu s'ouvrir", "The museum could not be opened", "No se ha podido abrir el museo")} (${echapper(e.message)}).</p>`; });
