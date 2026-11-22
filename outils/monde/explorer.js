// Explorer le musée autrement, sur le site : les artistes de A à Z (et leurs œuvres dans
// les collections du monde), les styles, genres, techniques et sujets (leurs œuvres dans les
// musées du monde), la chronologie siècle par siècle, les thèmes, et le jeu « Qui l'a fait ? »
// (le record est gardé dans le navigateur).

import { annee, carteOeuvre, chargerCatalogue, choixVue, echapper, lien, lienPage, nombre, plier, rangSiecle, renseignement, route, rubrique, t, titreSiecle, toutAfficher, toutes } from "./commun.js";
import { chargerSujets, nomsDesMusees, oeuvresDuSujet, oeuvresEnDirect, sujetsPublies } from "./direct.js";
import { ouvrirAuClic } from "./visionneuse.js";
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
  <div class="monde-onglets">${[["", t("Les artistes", "Artists", "Los artistas")], ["styles", t("Styles et genres", "Styles and genres", "Estilos y géneros")], ["chronologie", t("La chronologie", "Timeline", "La cronología")],
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

// Les œuvres des collections du monde de chaque artiste : liste.json (3,6 Mo compressés), lu
// seulement quand il le faut (la page d'un artiste) ou une fois la liste affichée, sans la
// retarder. La liste elle-même n'a besoin que de donnees/artistes.json (le nombre d'œuvres et
// une vignette par artiste) : sur un téléphone en 4G, 12 s d'attente devenaient quelques-unes.
let mondeCharge, mondeFini = false;
function chargerMonde() {
  mondeCharge ??= chargerArtistes().then(({ parCle }) => Promise.all([
    jsonGarde(lien("monde/donnees/liste.json")).catch(() => []),
    jsonGarde(lien("monde/donnees/index.json")).catch(() => ({ ids: [] })),
  ]).then(([liste, index]) => {
    for (const a of parCle.values()) a.monde = [];
    liste.forEach((l, i) => {
      const nom = l[1];
      if (!nom || nom === "Artiste inconnu") return;
      const cle = cleNom(nom);
      if (!parCle.has(cle)) parCle.set(cle, { nom, vie: "", style: "", bio: "", oeuvres: [], monde: [] });
      parCle.get(cle).monde.push({ l, id: index.ids[i] });
    });
    for (const a of parCle.values()) a.resume = null;
    mondeFini = true;
  }));
  return mondeCharge;
}

function chargerArtistes() {
  catalogueArtistes ??= Promise.all([
    jsonGarde(lien("monde/alias.json")).catch(() => ({})).then((a) => { ALIAS = a; }),
    // Le résumé des artistes des collections (outils/monde_web.py) : [nom, œuvres, id, vignette].
    jsonGarde(lien("monde/donnees/artistes.json")).catch(() => null),
    // L'annuaire : tous les artistes du domaine public dont un musée montre une œuvre
    // (outils/annuaire.py) : [nom, naissance, mort, œuvres].
    jsonGarde(lien("monde/annuaire.json"), 7 * 24 * 3600e3).catch(() => []),
  ]).then(([, resume, annuaire]) => {
    const parCle = new Map();
    for (const a of artistes) {
      const cle = cleNom(a.nom), deja = parCle.get(cle);
      if (deja) { deja.oeuvres.push(...a.oeuvres); if (a.bio.length > deja.bio.length) Object.assign(deja, { vie: a.vie, style: a.style, bio: a.bio }); }
      else parCle.set(cle, { ...a, oeuvres: [...a.oeuvres], monde: [] });
    }
    // Les artistes des collections du monde, en résumé (le détail : chargerMonde).
    for (const [nom, n, id, vu] of resume || []) {
      const cle = cleNom(nom);
      if (!parCle.has(cle)) parCle.set(cle, { nom, vie: "", style: "", bio: "", oeuvres: [], monde: [] });
      const a = parCle.get(cle);
      a.resume ??= { n: 0, premiere: { l: vu ? { 7: vu } : [], id } };
      a.resume.n += n;
    }
    for (const [nom, ne, mort, n] of annuaire) {
      const cle = cleNom(nom), deja = parCle.get(cle);
      const vie = ne && mort ? `${ne}–${mort}` : mort ? `${t("mort en", "died", "murió en")} ${mort}` : ne ? `${t("né en", "born", "nació en")} ${ne}` : "";
      if (deja) { deja.ailleurs = Math.max(deja.ailleurs || 0, n); deja.vie ||= vie; }
      else parCle.set(cle, { nom, vie, style: "", bio: "", oeuvres: [], monde: [], ailleurs: n });
    }
    const tous = [...parCle.values()].sort((a, b) => plier(a.nom).localeCompare(plier(b.nom)));
    return { tous, parCle, resume: !!resume };
  });
  return catalogueArtistes;
}

// Le nombre d'œuvres connues : celles d'ARTEON, ou celles que les musées montrent ailleurs.
const total = (a) => Math.max(a.oeuvres.length + (a.monde.length || a.resume?.n || 0), a.ailleurs || 0);
const lettreDe = (nom) => { const l = (plier(nom).replace(/^[^a-z0-9]+/, "")[0] || "#").toUpperCase(); return /[A-Z]/.test(l) ? l : "#"; };
const vignetteMonde = ({ l, id }) => l[7] || lien(`monde/v/${id}.webp`);
const vignetteDe = (a) => (a.oeuvres[0] ? lien(a.oeuvres[0].w.image) : a.monde[0] ? vignetteMonde(a.monde[0])
  : a.resume ? vignetteMonde(a.resume.premiere) : "");

async function listeArtistes(choisie) {
  racine.innerHTML = menu("") + `<p class="monde-chargement">${t("Ouverture du catalogue des artistes…", "Opening the catalogue of artists…", "Abriendo el catálogo de artistas…")}</p>`;
  const { tous, resume } = await chargerArtistes();
  // Sans le résumé (un site d'avant donnees/artistes.json), les œuvres d'abord, comme avant.
  if (!resume) await chargerMonde();
  // Entre-temps on est parti vers une autre page de l'Explorer : elle s'affiche d'elle-même.
  // (Une adresse inconnue, elle, montre la liste : elle ne reste pas sur « Ouverture… ».)
  if (["artiste", "chronologie", "themes", "jeu", "styles", "style"].includes(route().type)) return;
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
        : a.monde.length || a.resume ? t("Dans les collections du monde", "In the world collections", "En las colecciones del mundo")
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
    // Tous les artistes de la lettre, d'un coup (des milliers : par paquets, sans bouton).
    const ici = tous.filter((a) => lettreDe(a.nom) === lettre);
    zone.innerHTML = `<h2 class="monde-lettre">${lettre} <small class="monde-note">${nombre(ici.length)} ${ici.length > 1 ? t("artistes", "artists", "artistas") : t("artiste", "artist", "artista")}</small></h2><div class="monde-artistes" id="lettre-liste"></div>`;
    toutAfficher(document.getElementById("lettre-liste"), ici, ligne);
  };
  document.getElementById("filtre").addEventListener("input", (e) => dessiner(e.target.value));
  dessiner("");
  // La liste est là : les œuvres des collections arrivent maintenant, pour que la page d'un
  // artiste s'ouvre aussitôt.
  setTimeout(() => chargerMonde(), 1500);
}

/** Une carte des collections du monde : la vignette, le titre, la date et le musée. */
const carteMonde = ({ l, id }) => `<a class="monde-carte" href="${lienPage(`monde/#/oeuvre/${id}`)}"><img loading="lazy" decoding="async" src="${vignetteMonde({ l, id })}" alt=""><b>${echapper(l[0])}</b><span>${echapper([l[2], t("collections du monde", "world collections", "colecciones del mundo")].filter(Boolean).join(" · "))}</span></a>`;

async function artiste(nom) {
  racine.innerHTML = `<p class="monde-chargement">${t("Ouverture du catalogue…", "Opening the catalogue…", "Abriendo el catálogo…")}</p>`;
  const { parCle } = await chargerArtistes();
  if (route().type !== "artiste" || route().valeur !== nom) return;
  // En même temps : ses œuvres des collections du monde (liste.json, souvent déjà là, voir
  // listeArtistes) et celles des musées, en direct. La page s'affiche sans les attendre.
  const monde = chargerMonde();
  const direct = oeuvresEnDirect(nom, cleNom, ALIAS);
  direct.catch(() => {});
  // Un artiste qu'ARTEON ne montre pas encore : son catalogue vient tout entier des musées.
  const a = parCle.get(cleNom(nom)) || { nom, vie: "", style: "", bio: "", oeuvres: [], monde: [] };
  // Le catalogue complet, du plus ancien au plus récent : les œuvres des expositions et
  // celles des collections du monde, ensemble ; puis, en direct, celles des musées.
  const catalogue = () => [
    ...a.oeuvres.map(({ ex, w }) => ({ y: annee(w.year), html: carteOeuvre(ex, w, [w.year, ex.short_name || ex.title].filter(Boolean).join(" · ")) })),
    ...a.monde.map((m) => ({ y: m.l[3], html: carteMonde(m) })),
  ].sort((x, y) => (x.y ?? 1e9) - (y.y ?? 1e9));
  let toutes = catalogue();
  const enRoute = !mondeFini && a.resume?.n;
  const note = () => [
    a.oeuvres.length ? `${nombre(a.oeuvres.length)} ${t("dans les expositions du musée", "in the museum's exhibitions", "en las exposiciones del museo")}` : "",
    a.monde.length ? `${nombre(a.monde.length)} ${t("dans les collections du monde", "in the world collections", "en las colecciones del mundo")}` : "",
    toutes.length ? t("de la plus ancienne à la plus récente", "from oldest to most recent", "de la más antigua a la más reciente")
      : t("Ses œuvres viennent des musées, en direct", "Their works come live from the museums", "Sus obras llegan de los museos, en directo"),
  ].filter(Boolean).join(" · ");
  const titre = (n) => `${t("Le catalogue complet : ", "Complete catalogue: ", "El catálogo completo: ")}${oeuvresN(n)}`;
  const enDirect = t("Dans les musées, en direct", "Live from the museums", "En los museos, en directo");
  racine.innerHTML = `<a class="monde-retour" href="#/lettre/${encodeURIComponent(lettreDe(a.nom))}">← ${t("Les artistes", "Artists", "Los artistas")}</a>
    <h1 class="monde-titre">${echapper(a.nom)}</h1>
    ${a.vie ? `<dl class="facts">${renseignement(t("Sa vie en dates", "Life dates", "Fechas"), a.vie)}</dl>` : ""}
    ${a.bio || a.style ? `<div class="folds">${a.bio ? rubrique(t("Sa vie", "Life", "Su vida"), `<p>${echapper(a.bio)}</p>`, true) : ""}${a.style ? rubrique(t("Sa manière", "Technique", "Su estilo"), `<p>${echapper(a.style)}</p>`) : ""}</div>` : ""}
    <section class="monde-section"><h2 id="catalogue-titre">${titre(toutes.length + (enRoute ? a.resume.n : 0))}</h2>
      <p class="monde-note" id="catalogue-note">${note()}</p>
      ${enRoute ? `<p class="monde-chargement" id="catalogue-attente">${t(`Ses ${nombre(a.resume.n)} œuvres des collections du monde arrivent…`, `Their ${nombre(a.resume.n)} works from the world collections are on their way…`, `Sus ${nombre(a.resume.n)} obras de las colecciones del mundo están llegando…`)}</p>` : ""}
      <div class="monde-grille" id="catalogue-grille">${toutes.map((o) => o.html).join("")}</div></section>
    <section class="monde-section" id="en-direct"><h2>${enDirect}</h2>
      <p class="monde-chargement">${t("Recherche de toutes ses œuvres libres de droits dans les musées du monde…", "Searching the world's museums for all their copyright-free works…", "Buscando todas sus obras libres de derechos en los museos del mundo…")}</p></section>`;
  // Mosaïque ou cartes : pour voir toutes ses œuvres d'un coup d'œil et choisir.
  document.getElementById("catalogue-titre").nextElementSibling.after(choixVue());
  window.scrollTo(0, 0);
  const zone = document.getElementById("en-direct");
  // Les œuvres des collections du monde arrivées : le catalogue se complète.
  await monde.catch(() => {});
  if (route().type !== "artiste" || route().valeur !== nom) return;
  if (enRoute) {
    toutes = catalogue();
    document.getElementById("catalogue-attente")?.remove();
    document.getElementById("catalogue-titre").textContent = titre(toutes.length);
    document.getElementById("catalogue-note").textContent = note();
    document.getElementById("catalogue-grille").innerHTML = toutes.map((o) => o.html).join("");
  }
  try {
    const deja = new Set(a.monde.map((m) => m.id));
    const r = await direct;
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
      <div class="monde-grille" id="direct-grille"></div>`
      : `<h2>${enDirect}</h2><p class="monde-note">${t("Aucune autre œuvre libre de droits dans les collections ouvertes des musées.", "No other copyright-free works in the museums' open collections.", "Ninguna otra obra libre de derechos en las colecciones abiertas de los museos.")}</p>`;
    // Toutes ses œuvres, d'un coup (des milliers parfois : par paquets, sans bouton).
    const grille = document.getElementById("direct-grille");
    // Un clic ouvre l'œuvre en grand dans ARTEON (visionneuse.js) ; le lien mène chez le musée sans JavaScript.
    const carte = (o) => `<a class="monde-carte" data-oeuvre="${echapper(o.id)}" href="${echapper(o.lien)}" target="_blank" rel="noopener"><img loading="lazy" decoding="async" src="${echapper(o.vignette)}" alt=""><b>${echapper(o.titre || t("Sans titre", "Untitled", "Sin título"))}</b><span>${echapper([o.date, o.fournisseur && !o.fournisseur.startsWith("http") ? `${o.fournisseur} · ${(musee[o.musee] || "").split(" (")[0]}` : musee[o.musee]].filter(Boolean).join(" · "))}</span></a>`;
    if (grille) {
      toutAfficher(grille, nouvelles, carte);
      ouvrirAuClic(grille, nouvelles, musee);
    }
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

// Les styles et mouvements, les genres, les techniques et les sujets représentés
// (outils/monde/sujets.json) : pour chacun, ses œuvres libres de droits dans les musées du
// monde, les plus connues d'abord. Le catalogue de chaque sujet est publié avec le site
// (api/sujets/) ; s'il ne l'est pas encore, le navigateur le demande lui-même.
const nomSujet = (s) => t(s.fr, s.en, s.es);
const pasDeReponse = () => t("Les musées n'ont pas répondu. Réessayer plus tard.", "The museums did not respond. Please try again later.", "Los museos no han respondido. Vuelva a intentarlo más tarde.");

// Toutes les catégories de Wikidata (outils/sujets_wikidata.py) : des milliers. D'abord toutes
// confondues, des plus riches aux moins riches ; un bouton par famille (styles, genres,
// techniques…) ne montre qu'elle. Les premières, puis « Tout afficher » ; la recherche porte
// sur toutes celles du bouton choisi.
const PREMIERS = 24;

async function listeSujets() {
  racine.innerHTML = menu("styles") + `<p class="monde-chargement">${t("Ouverture des styles et des genres…", "Opening the styles and genres…", "Abriendo los estilos y los géneros…")}</p>`;
  let table, publies;
  try { [table, publies] = await Promise.all([chargerSujets(), sujetsPublies()]); } catch {
    if (route().type === "styles") racine.innerHTML = menu("styles") + `<p class="monde-chargement">${pasDeReponse()}</p>`;
    return;
  }
  if (route().type !== "styles") return;
  const n = table.familles.reduce((somme, f) => somme + f.sujets.length, 0);
  const tuile = (s) => {
    const p = publies[s.cle], image = p?.image || s.image, total = p?.total || s.n;
    return `<a class="monde-sujet${image ? "" : " sans-image"}" href="#/style/${s.cle}">${image ? `<img loading="lazy" decoding="async" src="${echapper(image)}" alt="">` : ""}<b>${echapper(nomSujet(s))}</b>${total ? `<span>${oeuvresN(total)}</span>` : ""}</a>`;
  };
  const tous = table.familles.flatMap((f) => f.sujets.map((s) => ({ s, f, plie: plier(`${s.fr} ${s.en} ${s.es}`) })));
  // Toutes les catégories confondues : chaque sujet une fois (un même sujet peut être dans deux
  // familles), des plus riches aux moins riches.
  const vus = new Set();
  const confondus = tous.filter((x) => !vus.has(x.s.cle) && vus.add(x.s.cle))
    .sort((a, b) => ((publies[b.s.cle]?.total || b.s.n || 0) - (publies[a.s.cle]?.total || a.s.n || 0)));
  const groupes = [{ cle: "toutes", fr: "Toutes", en: "All", es: "Todas", liste: confondus },
    ...table.familles.map((f) => ({ ...f, liste: tous.filter((x) => x.f === f) }))];
  let choisi = groupes[0];
  racine.innerHTML = menu("styles") + `
    <p class="monde-lead">${t(
      `${nombre(n)} styles, genres, techniques, sujets et musées : toutes les catégories de Wikidata qui ont au moins douze œuvres libres de droits, de l'art gothique au Bauhaus, du portrait à la nature morte, de la fresque à l'estampe. Pour chacune, ses œuvres, les plus connues d'abord.`,
      `${nombre(n)} styles, genres, techniques, subjects and museums: every Wikidata category with at least twelve copyright-free works, from Gothic art to the Bauhaus, from portraits to still lifes, from frescoes to prints. For each, its works, best known first.`,
      `${nombre(n)} estilos, géneros, técnicas, temas y museos: todas las categorías de Wikidata con al menos doce obras libres de derechos, del arte gótico a la Bauhaus, del retrato al bodegón, del fresco a la estampa. Para cada una, sus obras, las más conocidas primero.`)}</p>
    <div class="monde-barre"><input type="search" id="filtre" placeholder="${t("Un style, un genre, un sujet, un musée…", "A style, a genre, a subject, a museum…", "Un estilo, un género, un tema, un museo…")}" aria-label="${t("Chercher une catégorie", "Search for a category", "Buscar una categoría")}"></div>
    <div class="monde-themes" id="familles">${groupes.map((g) => `<button type="button" data-famille="${g.cle}" aria-pressed="${g === choisi}">${echapper(nomSujet(g))} <span>${nombre(g.liste.length)}</span></button>`).join("")}</div>
    <div id="sujets-liste"></div>`;
  const liste = document.getElementById("sujets-liste"), filtre = document.getElementById("filtre");
  const montrer = () => {
    const mots = plier(filtre.value).split(/\s+/).filter(Boolean);
    for (const b of document.querySelectorAll("#familles button")) b.setAttribute("aria-pressed", String(b.dataset.famille === choisi.cle));
    if (!mots.length) {
      // Les premières ; « Tout afficher » déplie le reste, par paquets (des milliers).
      const l = choisi.liste;
      liste.innerHTML = `<div class="monde-sujets">${l.slice(0, PREMIERS).map((x) => tuile(x.s)).join("")}</div>
        ${l.length > PREMIERS ? `<button type="button" class="monde-plus" data-deplier>${t(`Tout afficher (${nombre(l.length)})`, `Show all (${nombre(l.length)})`, `Mostrar todo (${nombre(l.length)})`)}</button>` : ""}`;
      return;
    }
    // Les noms qui commencent par le mot cherché d'abord, puis les plus riches.
    const trouves = choisi.liste.filter((x) => mots.every((m) => x.plie.includes(m)))
      .sort((a, b) => (b.plie.startsWith(mots[0]) - a.plie.startsWith(mots[0])) || ((b.s.n || 0) - (a.s.n || 0)));
    liste.innerHTML = trouves.length
      ? `<p class="monde-compte">${nombre(trouves.length)} ${trouves.length > 1 ? t("catégories", "categories", "categorías") : t("catégorie", "category", "categoría")}${trouves.length > 300 ? ` ${t("(les 300 premières)", "(the first 300)", "(las 300 primeras)")}` : ""}</p>
        <div class="monde-sujets">${trouves.slice(0, 300).map((x) => tuile(x.s)).join("")}</div>`
      : `<p class="monde-chargement">${t("Aucune catégorie de ce nom.", "No category by that name.", "Ninguna categoría con ese nombre.")}</p>`;
  };
  document.getElementById("familles").addEventListener("click", (e) => {
    const b = e.target.closest("button[data-famille]");
    if (!b) return;
    choisi = groupes.find((g) => g.cle === b.dataset.famille) || groupes[0];
    montrer();
  });
  liste.addEventListener("click", (e) => {
    const b = e.target.closest("button[data-deplier]");
    if (!b) return;
    toutAfficher(b.previousElementSibling, choisi.liste.slice(PREMIERS), (x) => tuile(x.s));
    b.remove();
  });
  filtre.addEventListener("input", montrer);
  montrer();
  window.scrollTo(0, 0);
}

async function pageSujet(cle) {
  racine.innerHTML = `<p class="monde-chargement">${t("Ouverture…", "Opening…", "Abriendo…")}</p>`;
  const table = await chargerSujets().catch(() => null);
  if (route().type !== "style" || route().valeur !== cle) return;
  const famille = table?.familles.find((f) => f.sujets.some((s) => s.cle === cle));
  const sujet = famille?.sujets.find((s) => s.cle === cle);
  const retour = `<a class="monde-retour" href="#/styles">← ${t("Styles et genres", "Styles and genres", "Estilos y géneros")}</a>`;
  if (!sujet) {
    racine.innerHTML = `${retour}<p class="monde-chargement">${table ? t("Ce style n'est pas dans la liste.", "This style is not in the list.", "Este estilo no está en la lista.") : pasDeReponse()}</p>`;
    return;
  }
  racine.innerHTML = `${retour}
    <h1 class="monde-titre">${echapper(nomSujet(sujet))}</h1>
    <p class="monde-note">${echapper(nomSujet(famille))}</p>
    <section class="monde-section" id="sujet-oeuvres"><p class="monde-chargement">${t("Recherche de ses œuvres libres de droits dans les musées du monde…", "Searching the world's museums for its copyright-free works…", "Buscando sus obras libres de derechos en los museos del mundo…")}</p></section>`;
  window.scrollTo(0, 0);
  const zone = document.getElementById("sujet-oeuvres");
  try {
    const r = await oeuvresDuSujet(cle);
    if (route().type !== "style" || route().valeur !== cle) return;
    const musee = await nomsDesMusees().catch(() => ({}));
    if (!r.total) {
      zone.innerHTML = `<p class="monde-note">${t("Aucune œuvre libre de droits trouvée pour l'instant.", "No copyright-free works found for now.", "Ninguna obra libre de derechos encontrada por ahora.")}</p>`;
      return;
    }
    zone.innerHTML = `<h2>${oeuvresN(r.total)}</h2>
      <p class="monde-note" id="sujet-note">${t("Les plus connues d'abord · Domaine public (artiste mort en 1955 ou avant, œuvre d'avant 1931) · images et fiches chez les musées",
        "Best known first · Public domain (artist died in 1955 or earlier, work made before 1931) · images and records hosted by the museums",
        "Las más conocidas primero · Dominio público (artista fallecido en 1955 o antes, obra anterior a 1931) · imágenes y fichas en los museos")}</p>
      <div class="monde-grille" id="sujet-grille"></div>`;
    // Mosaïque ou cartes, comme sur la page d'un artiste.
    document.getElementById("sujet-note").after(choixVue());
    const grille = document.getElementById("sujet-grille");
    const carte = (o) => `<a class="monde-carte" data-oeuvre="${echapper(o.id)}" href="${echapper(o.lien)}" target="_blank" rel="noopener"><img loading="lazy" decoding="async" src="${echapper(o.vignette)}" alt=""><b>${echapper(o.titre || t("Sans titre", "Untitled", "Sin título"))}</b><span>${echapper([o.artiste, o.date].filter(Boolean).join(" · "))}</span></a>`;
    toutAfficher(grille, r.oeuvres, carte);
    ouvrirAuClic(grille, r.oeuvres, musee);
  } catch {
    zone.innerHTML = `<p class="monde-note">${pasDeReponse()}</p>`;
  }
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
  if (r.type === "styles") return listeSujets();
  if (r.type === "style") return pageSujet(r.valeur);
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
