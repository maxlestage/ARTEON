// Les collections du monde, sur le site : des dizaines de milliers d'œuvres du domaine
// public (Chicago, Cleveland, le Met, Washington, le Smithsonian). Les fiches arrivent en morceaux
// (donnees/liste-c.json pour parcourir et chercher, donnees/fiches-NN.json pour le détail),
// les vignettes viennent du site (v/<œuvre>.jpg), l'image en grand du musée qui la publie.
// Écrit par outils/monde_web.py dans monde/ ; adresses en #/… (une seule page).
import { chargerCatalogue, langue, lien, lienPage, nombre, t, titreSiecle, toutAfficher } from "./commun.js";
import { chargerCollections } from "./liste.js";
import { jsonGarde } from "./memoire.js";

const racine = document.getElementById("monde");
const etat = { index: null, liste: null, plies: null, fiches: new Map(), vues: [], montrees: 0 };

const echapper = (t) => String(t ?? "").replace(/[&<>"']/g, (c) =>
  ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const plier = (t) => String(t ?? "").replace(/œ/g, "oe").replace(/Œ/g, "oe").replace(/æ/g, "ae")
  .normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
const cite = (s) => t(`« ${s} »`, `“${s}”`, `«${s}»`);
const oeuvresN = (n) => (n === 1 ? t("1 œuvre", "1 work", "1 obra") : `${nombre(n)} ${t("œuvres", "works", "obras")}`);

// Les catégories (outils/collections_monde.py) restent en français dans les données et les
// adresses ; seul leur nom affiché change de langue.
const CATEGORIES = {
  "Peinture": ["Painting", "Pintura"], "Dessin": ["Drawing", "Dibujo"], "Estampe": ["Print", "Estampa"],
  "Photographie": ["Photography", "Fotografía"], "Sculpture": ["Sculpture", "Escultura"],
  "Céramique": ["Ceramics", "Cerámica"], "Textile et costume": ["Textiles and costume", "Textiles e indumentaria"],
  "Mobilier": ["Furniture", "Mobiliario"], "Verre": ["Glass", "Vidrio"], "Bijoux": ["Jewellery", "Joyería"],
  "Armes et armures": ["Arms and armour", "Armas y armaduras"], "Livres et manuscrits": ["Books and manuscripts", "Libros y manuscritos"],
  "Monnaies et médailles": ["Coins and medals", "Monedas y medallas"], "Instruments de musique": ["Musical instruments", "Instrumentos musicales"],
  "Orfèvrerie et métal": ["Metalwork", "Orfebrería y metal"], "Autres objets": ["Other objects", "Otros objetos"],
  // Les catégories plus fines (outils/categories.py).
  "Estampe japonaise": ["Japanese prints", "Estampa japonesa"],
  "Affiche": ["Posters", "Carteles"],
  "Dessin d'architecture": ["Architectural drawings", "Dibujo de arquitectura"],
  "Miniature et enluminure": ["Miniatures and illumination", "Miniatura e iluminación"],
  "Rouleaux et paravents": ["Scrolls and screens", "Rollos y biombos"],
  "Icônes et retables": ["Icons and altarpieces", "Iconos y retablos"],
  "Aquarelle": ["Watercolour", "Acuarela"],
  "Pastel": ["Pastel", "Pastel"],
  "Éventails": ["Fans", "Abanicos"],
  "Masques": ["Masks", "Máscaras"],
  "Vannerie": ["Basketry", "Cestería"],
  "Mosaïques et vitraux": ["Mosaics and stained glass", "Mosaicos y vidrieras"],
  "Horlogerie": ["Clocks and watches", "Relojería"],
  "Art funéraire égyptien": ["Egyptian funerary art", "Arte funerario egipcio"],
  "Sceaux et amulettes": ["Seals and amulets", "Sellos y amuletos"],
  "Tapisseries et tapis": ["Tapestries and carpets", "Tapices y alfombras"],
  "Costume et mode": ["Costume and fashion", "Indumentaria y moda"],
  "Dentelle et broderie": ["Lace and embroidery", "Encaje y bordado"],
  "Vases grecs": ["Greek vases", "Vasos griegos"],
  "Porcelaine": ["Porcelain", "Porcelana"],
  "Laque": ["Lacquer", "Laca"],
  "Ivoire": ["Ivory", "Marfil"],
  "Jade et pierres dures": ["Jade and hardstones", "Jade y piedras duras"],
  "Sculpture en bronze": ["Bronze sculpture", "Escultura en bronce"],
  "Sculpture en marbre et en pierre": ["Marble and stone sculpture", "Escultura en mármol y piedra"],
  "Sculpture en bois": ["Wood sculpture", "Escultura en madera"],
  "Sculpture en terre cuite": ["Terracotta sculpture", "Escultura en terracota"],
};
const categorie = (c) => (CATEGORIES[c] ? t(c, ...CATEGORIES[c]) : c);
// « Artiste inconnu » : la valeur des données (et la clé des recherches), affichée dans la langue de la page.
const nomArtiste = (a) => (a === "Artiste inconnu" ? t(a, "Unknown artist", "Artista desconocido") : a);
// Les villes et licences des musées (index.json) sont écrites en français.
const localiser = (texte) => (langue === "fr" ? String(texte ?? "") : String(texte ?? "")
  .replace("États-Unis", t("États-Unis", "United States", "Estados Unidos"))
  .replace("Images et données", t("Images et données", "Images and data", "Imágenes y datos"))
  .replace(" ; textes de présentation", t(" ; textes de présentation", "; descriptive texts", "; textos de presentación")));
const rangSiecle = (y) => y <= -100000 ? -999999 : (y < 0 ? -(Math.floor((-y - 1) / 100) + 1) : Math.floor((y - 1) / 100) + 1);

// Une ligne de la liste (liste.js) : [titre, artiste, date, année, catégorie, musée, ratio].
const T = 0, A = 1, D = 2, Y = 3, C = 4, K = 5;

async function charger() {
  racine.innerHTML = `<p class="monde-chargement">${t("Ouverture des collections…", "Opening the collections…", "Abriendo las colecciones…")}</p>`;
  // La liste complète avec son index (liste-c.json, 5 Mo compressés) et son début
  // (liste-debut.json, quelques dizaines de Ko) : la page « Toutes les œuvres » s'affiche avec
  // le début et l'en-tête de l'index (quelques Ko, les identifiants du début seulement), et la
  // suite s'ajoute quand la liste complète est là. Une recherche, un filtre ou une fiche
  // l'attendent. La deuxième visite la trouve dans la mémoire du navigateur (memoire.js).
  const complete = chargerCollections();
  const prendreIndex = (index) => {
    etat.index = index;
    etat.ids = index.ids;
    etat.position = new Map(index.ids.map((id, i) => [id, i]));
  };
  // L'en-tête et le début de la liste, demandés ensemble.
  const accueil = !route().type;
  const enteteP = accueil ? jsonGarde(lien("monde/donnees/entete.json")).catch(() => null) : null;
  const debutP = accueil ? jsonGarde(lien("monde/donnees/liste-debut.json")).catch(() => null) : null;
  const entete = await enteteP;
  const debut = entete ? await Promise.race([complete.then(() => null), debutP]) : null;
  window.addEventListener("hashchange", afficher);
  // Une fiche ouverte directement (un lien, un favori) : elle s'affiche sans attendre la liste.
  if (route().type === "oeuvre") afficher();
  if (debut?.length && entete.idsDebut?.length >= debut.length) {
    prendreIndex({ ...entete, ids: entete.idsDebut });
    etat.liste = debut;
    etat.partielle = true;
    afficher();
  }
  const tout = await complete;
  prendreIndex(tout.index);
  etat.liste = tout.liste;
  etat.plies = null;
  const avant = debut?.length || 0;
  etat.partielle = false;
  const grille = document.getElementById("monde-grille");
  const r = route();
  if (etat.seule && r.type === "oeuvre" && r.valeur === etat.seule && document.getElementById("monde-voisins")) {
    // La fiche affichée sans la liste : il ne lui manquait que ses voisines et les œuvres du même artiste.
    const i = etat.position.get(etat.seule);
    etat.seule = null;
    document.getElementById("monde-voisins").innerHTML = voisines(i);
    document.getElementById("monde-memes").innerHTML = duMemeArtiste(i, etat.liste[i]);
  } else if (avant && !route().type && grille) {
    // Toujours sur « Toutes les œuvres » : la suite s'ajoute sous ce qui est affiché.
    etat.vues = [...etat.liste.keys()];
    toutAfficher(grille, etat.vues.slice(avant), carte);
  } else {
    afficher();
  }
}

function filtrer(route) {
  const L = etat.liste;
  const tous = [...L.keys()];
  if (route.type === "musee") { const k = etat.index.musees.findIndex((m) => m.cle === route.valeur); return tous.filter((i) => L[i][K] === k); }
  if (route.type === "categorie") return tous.filter((i) => L[i][C] === route.valeur);
  if (route.type === "siecle") { const r = Number(route.valeur); return tous.filter((i) => L[i][Y] != null && rangSiecle(L[i][Y]) === r); }
  if (route.type === "artiste") return tous.filter((i) => L[i][A] === route.valeur);
  if (route.type === "exposition") return etat.expositions?.[route.valeur]?.indices ?? [];
  if (route.type === "recherche") {
    const mots = plier(route.valeur).split(/\s+/).filter(Boolean);
    if (!mots.length) return tous;
    if (!etat.plies) etat.plies = L.map((l) => plier(`${l[T]} ${l[A]} ${l[D]} ${l[C]} ${langue === "fr" ? "" : categorie(l[C])} ${etat.index.musees[l[K]].nom}`));
    return tous.filter((i) => mots.every((m) => etat.plies[i].includes(m)));
  }
  return tous;
}

function route() {
  const [, type, ...reste] = decodeURIComponent(location.hash.replace(/^#\/?/, "#/")).split("/");
  return { type: type || "", valeur: reste.join("/") };
}

function afficher() {
  const r = route();
  // Une recherche, un filtre ou une fiche pendant que la liste complète arrive : elle les
  // affichera (charger).
  etat.seule = null;
  if ((!etat.liste || etat.partielle) && r.type === "oeuvre") return fiche(r.valeur);
  if (!etat.liste || (etat.partielle && r.type)) {
    racine.innerHTML = `<p class="monde-chargement">${t("Ouverture de toutes les œuvres…", "Opening all the works…", "Abriendo todas las obras…")}</p>`;
    return;
  }
  if (r.type === "oeuvre") return fiche(r.valeur);
  // Les œuvres liées à une exposition : chargées à la première demande.
  if (r.type === "exposition" && !etat.expositions) {
    racine.innerHTML = `<p class="monde-chargement">${t("Ouverture de l'exposition…", "Opening the exhibition…", "Abriendo la exposición…")}</p>`;
    // Le nom de l'exposition vient du catalogue de la langue de la page (expositions.json est en français).
    Promise.all([
      jsonGarde(lien("monde/donnees/expositions.json")).catch(() => ({})),
      langue === "fr" ? null : chargerCatalogue().catch(() => null),
    ]).then(([e, c]) => {
      for (const ex of c?.exhibitions || []) if (e[ex.slug]) e[ex.slug].nom = ex.short_name || ex.title || e[ex.slug].nom;
      etat.expositions = e;
      if (route().type === "exposition") afficher();
    });
    return;
  }
  const vues = filtrer(r);
  etat.vues = vues;
  etat.montrees = 0;
  const ix = etat.index;
  const titre = {
    musee: ix.musees.find((m) => m.cle === r.valeur)?.nom,
    categorie: categorie(r.valeur), siecle: ix.siecles.some((s) => String(s[0]) === r.valeur) ? titreSiecle(Number(r.valeur)) : "",
    artiste: nomArtiste(r.valeur), recherche: r.valeur ? cite(r.valeur) : "",
    exposition: etat.expositions?.[r.valeur] ? `${t("Exposition", "Exhibition", "Exposición")} ${cite(etat.expositions[r.valeur].nom)}` : "",
  }[r.type] || t("Toutes les œuvres", "All works", "Todas las obras");
  racine.innerHTML = `
    <h1 class="monde-titre">${t("Les collections du monde", "World collections", "Las colecciones del mundo")}</h1>
    <p class="monde-lead">${t(
      `${nombre(ix.total)} œuvres du domaine public, choisies dans les collections ouvertes
      de ${ix.musees.length} grands musées. Les images en grand viennent des musées eux-mêmes (CC0).`,
      `${nombre(ix.total)} public-domain works, selected from the open collections
      of ${ix.musees.length} major museums. Full-size images come from the museums themselves (CC0).`,
      `${nombre(ix.total)} obras de dominio público, elegidas en las colecciones abiertas
      de ${ix.musees.length} grandes museos. Las imágenes en grande proceden de los propios museos (CC0).`)}</p>
    <div class="monde-musees">${ix.musees.map((m) => `
      <a class="monde-musee${r.type === "musee" && r.valeur === m.cle ? " actif" : ""}" style="--couleur:${m.couleur}" href="#/musee/${m.cle}">
        <strong>${echapper(m.nom)}</strong><span>${echapper(localiser(m.ville))} · ${oeuvresN(m.oeuvres)}</span></a>`).join("")}</div>
    <div class="monde-barre">
      <input type="search" id="monde-q" placeholder="${t("Un titre, un artiste, une technique, une année…", "A title, an artist, a medium, a year…", "Un título, un artista, una técnica, un año…")}" value="${r.type === "recherche" ? echapper(r.valeur) : ""}" aria-label="${t("Chercher", "Search", "Buscar")}">
      <select id="monde-cat" aria-label="${t("Catégorie", "Category", "Categoría")}"><option value="">${t("Toutes les catégories", "All categories", "Todas las categorías")}</option>
        ${ix.categories.map(([c, n]) => `<option${r.type === "categorie" && r.valeur === c ? " selected" : ""} value="${echapper(c)}">${echapper(categorie(c))} (${nombre(n)})</option>`).join("")}</select>
      <select id="monde-siecle" aria-label="${t("Siècle", "Century", "Siglo")}"><option value="">${t("Tous les siècles", "All centuries", "Todos los siglos")}</option>
        ${ix.siecles.map(([rang, , n]) => `<option${r.type === "siecle" && r.valeur === String(rang) ? " selected" : ""} value="${rang}">${echapper(titreSiecle(rang))} (${nombre(n)})</option>`).join("")}</select>
      <button id="monde-hasard" type="button">🎲 ${t("Au hasard", "Surprise me", "Al azar")}</button>
      ${r.type ? `<a class="monde-lien" href="#/">${t("Tout voir", "See all", "Ver todo")}</a>` : ""}
    </div>
    <p class="monde-compte"><strong>${echapper(titre)}</strong> · ${oeuvresN(etat.partielle ? ix.total : vues.length)}${r.type === "exposition"
      ? ` ${t("liées dans les collections du monde, de la plus ancienne à la plus récente", "related in the world collections, from oldest to most recent", "relacionadas en las colecciones del mundo, de la más antigua a la más reciente")} · <a href="${lienPage(`${encodeURIComponent(r.valeur)}/`)}">← ${t("la visite de l'exposition", "visit the exhibition", "la visita de la exposición")}</a>` : ""}</p>
    <div class="monde-grille" id="monde-grille"></div>`;
  let attente;
  const q = document.getElementById("monde-q");
  q.addEventListener("input", () => {
    clearTimeout(attente);
    // Pas si l'on a quitté la page entre-temps (une œuvre touchée aussitôt après avoir tapé).
    attente = setTimeout(() => { if (q.isConnected) location.hash = q.value.trim() ? `#/recherche/${encodeURIComponent(q.value.trim())}` : "#/"; }, 350);
  });
  if (r.type === "recherche") { q.focus(); q.setSelectionRange(q.value.length, q.value.length); }
  document.getElementById("monde-cat").addEventListener("change", (e) => { location.hash = e.target.value ? `#/categorie/${encodeURIComponent(e.target.value)}` : "#/"; });
  document.getElementById("monde-siecle").addEventListener("change", (e) => { location.hash = e.target.value ? `#/siecle/${e.target.value}` : "#/"; });
  document.getElementById("monde-hasard").addEventListener("click", () => {
    location.hash = `#/oeuvre/${etat.ids[Math.floor(Math.random() * etat.ids.length)]}`;
  });
  // Toutes les œuvres de la vue, sans bouton (au-delà de 12 000, la suite arrive en descendant).
  toutAfficher(document.getElementById("monde-grille"), vues, carte);
  if (!vues.length) {
    document.getElementById("monde-grille").insertAdjacentHTML("afterend", `<p class="monde-note">${t(
      "Aucune œuvre ne correspond. Essayez un autre mot (les titres des musées sont souvent en anglais), ou cherchez un artiste dans l'Explorer.",
      "No work matches. Try another word, or look for an artist in Explore.",
      "Ninguna obra coincide. Pruebe con otra palabra (los títulos de los museos suelen estar en inglés) o busque un artista en Explorar.")}
      <a href="${lienPage("explorer/")}">${t("Ouvrir l'Explorer", "Open Explore", "Abrir Explorar")} →</a></p>`);
  }
}

function carte(i) {
  const l = etat.liste[i], id = etat.ids[i];
  const ligne = [nomArtiste(l[A]), l[D]].filter(Boolean).join(" · ");
  return `<a class="monde-carte" href="#/oeuvre/${id}"><img loading="lazy" decoding="async" src="${l[7] || lien(`monde/v/${id}.webp`)}" alt=""><b>${echapper(l[T])}</b><span>${echapper(ligne)}</span></a>`;
}

async function detail(i, parFichier = etat.index.parFichier) {
  const part = Math.floor(i / parFichier);
  if (!etat.fiches.has(part)) {
    const nom = lien(`monde/donnees/fiches-${String(part).padStart(3, "0")}.json`);
    etat.fiches.set(part, jsonGarde(nom));
  }
  return (await etat.fiches.get(part))[i % parFichier];
}

// Une œuvre sans la liste complète (5 Mo compressés) : donnees/par-oeuvre/NNN.json
// (outils/monde_web.py, par_oeuvre) range les œuvres en PAQUETS fichiers de quelques Ko,
// d'après leur identifiant (le même calcul que paquet_de). Une œuvre : son rang dans la liste,
// puis sa ligne. Avec l'en-tête de l'index (les musées), c'est tout ce qu'il faut à sa fiche.
const PAQUETS = 512;
function paquetDe(cle) {
  let h = 0;
  for (let i = 0; i < cle.length; i++) h = (h * 31 + cle.charCodeAt(i)) % 65521;
  return String(h % PAQUETS).padStart(3, "0");
}
const seules = new Map();
function oeuvreSeule(id) {
  if (!seules.has(id)) {
    seules.set(id, Promise.all([
      jsonGarde(lien("monde/donnees/entete.json")),
      jsonGarde(lien(`monde/donnees/par-oeuvre/${paquetDe(id)}.json`)),
    ]).then(([index, p]) => {
      if (!p.oeuvres[id]) return null;
      const [i, ...l] = p.oeuvres[id];
      if (l[7] && !l[7].startsWith("http")) l[7] = p.gabarits[l[K]].replaceAll("{}", l[7]);
      return { index, i, l };
    }).catch(() => null));
  }
  return seules.get(id);
}

// La précédente et la suivante dans la vue, et douze œuvres du même artiste : avec toute la liste.
function voisines(i) {
  const vues = etat.vues.length ? etat.vues : [...etat.liste.keys()];
  const ici = vues.indexOf(i);
  const voisin = (d) => (ici >= 0 && vues[ici + d] != null ? etat.ids[vues[ici + d]] : null);
  const avant = voisin(-1), apres = voisin(1);
  return `${avant ? `<a href="#/oeuvre/${avant}">‹ ${t("Précédente", "Previous", "Anterior")}</a>` : ""}${apres ? `<a href="#/oeuvre/${apres}">${t("Suivante", "Next", "Siguiente")} ›</a>` : ""}`;
}
function duMemeArtiste(i, l) {
  const memes = l[A] && l[A] !== "Artiste inconnu"
    ? [...etat.liste.keys()].filter((j) => j !== i && etat.liste[j][A] === l[A]).slice(0, 12) : [];
  return memes.length ? `<section class="monde-section"><h2>${t("Du même artiste", "By the same artist", "Del mismo artista")}</h2><div class="monde-grille">${memes.map(carte).join("")}</div></section>` : "";
}

async function fiche(id) {
  // Pendant que la liste complète arrive : la fiche avec le petit fichier de l'œuvre ; ses
  // voisines et les œuvres du même artiste s'ajoutent quand la liste est là (charger).
  let seule = null;
  if (!etat.liste || etat.partielle) {
    racine.innerHTML = `<p class="monde-chargement">${t("Ouverture de l'œuvre…", "Opening the work…", "Abriendo la obra…")}</p>`;
    window.scrollTo(0, 0);
    seule = await oeuvreSeule(id);
    const r = route();
    if ((etat.liste && !etat.partielle) || r.type !== "oeuvre" || r.valeur !== id) return;
    if (!seule) {
      racine.innerHTML = `<p class="monde-chargement">${t("Ouverture de toutes les œuvres…", "Opening all the works…", "Abriendo todas las obras…")}</p>`;
      return;
    }
  }
  const i = seule ? seule.i : etat.position.get(id);
  if (i == null) { racine.innerHTML = `<p class="monde-chargement">${t("Cette œuvre n'est pas dans les collections.", "This work is not in the collections.", "Esta obra no está en las colecciones.")} <a href="#/">${t("Retour", "Back", "Volver")}</a></p>`; return; }
  const l = seule ? seule.l : etat.liste[i], index = seule ? seule.index : etat.index, m = index.musees[l[K]];
  if (!seule) racine.innerHTML = `<p class="monde-chargement">…</p>`;
  const f = await detail(i, index.parFichier);
  if (route().type !== "oeuvre" || route().valeur !== id) return;
  // La liste complète est arrivée entre-temps : c'est elle qui affiche la fiche (charger).
  if (seule && etat.liste && !etat.partielle) return;
  etat.seule = seule ? id : null;
  const faits = [[t("Date", "Date", "Fecha"), l[D]], [t("Technique", "Medium", "Técnica"), f.m], [t("Mesures", "Dimensions", "Medidas"), f.s],
    [t("Catégorie", "Category", "Categoría"), categorie(l[C])], [t("Origine", "Origin", "Origen"), f.o],
    [t("Musée", "Museum", "Museo"), `${m.nom}, ${localiser(m.ville)}`], [t("Où la voir", "Where to see it", "Dónde verla"), f.g]].filter(([, v]) => v);
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← ${t("Toutes les œuvres", "All works", "Todas las obras")}</a>
    <div class="monde-fiche">
      <div class="monde-image"><img id="monde-img" src="${l[7] || lien(`monde/v/${id}.webp`)}" alt="${echapper(l[T])}"${l[6] > 0
        ? ` style="aspect-ratio:${(1 / l[6]).toFixed(3)};width:min(100%, calc(78vh * ${(1 / l[6]).toFixed(3)}))"` : ""}></div>
      <div>
        <h1>${echapper(l[T])}</h1>
        <p class="monde-artiste"><a href="${lienPage(`explorer/#/artiste/${encodeURIComponent(l[A])}`)}" title="${t("Le catalogue complet de l'artiste", "The artist's complete catalogue", "El catálogo completo del artista")}">${echapper(nomArtiste(l[A]))}</a>${f.v ? ` · ${echapper(f.v)}` : ""}</p>
        <dl class="monde-cartel">${faits.map(([k, v]) => `<dt>${k}</dt><dd>${echapper(v)}</dd>`).join("")}</dl>
        ${f.x ? `<div class="folds"><details class="fold" open><summary><span>${t("Description", "Description", "Descripción")}</span><span class="fold-sign" aria-hidden="true"></span></summary>
          <div class="fold-body monde-texte"><p>${echapper(f.x)}</p>${m.cle === "chicago" ? `<p class="monde-note">${t("Texte : Art Institute of Chicago, CC BY 4.0.", "Text: Art Institute of Chicago, CC BY 4.0.", "Texto: Art Institute of Chicago, CC BY 4.0.")}</p>` : ""}</div></details></div>` : ""}
        <div class="monde-nav">
          <span id="monde-voisins" style="display:contents">${seule ? "" : voisines(i)}</span>
          ${f.w ? `<a href="${echapper(f.w)}" target="_blank" rel="noopener">${t("La fiche du musée", "On the museum's website", "La ficha del museo")} ↗</a>` : ""}
        </div>
        <p class="monde-note">${echapper(localiser(m.licence))}</p>
      </div>
    </div>
    <div id="monde-memes">${seule ? "" : duMemeArtiste(i, l)}</div>`;
  window.scrollTo(0, 0);
  if (f.u) {
    const grande = new Image();
    grande.onload = () => { const img = document.getElementById("monde-img"); if (img && route().valeur === id) img.src = f.u; };
    grande.src = f.u;
  }
}

charger().catch((e) => {
  racine.innerHTML = `<p class="monde-chargement">${t("Les collections n'ont pas pu s'ouvrir", "The collections could not be opened", "No se han podido abrir las colecciones")} (${echapper(e.message)}).</p>`;
});
