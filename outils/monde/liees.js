// Sur la page « Œuvres » d'une exposition : toutes les œuvres. Après celles de la visite 3D
// (écrites dans la page) viennent :
//  1. toutes les œuvres liées des collections du monde (monde/donnees/expo-<nom>.json,
//     écrit par outils/monde_web.py : [identifiant, titre, artiste, date, vignette ou rien]) ;
//  2. pour chaque artiste de l'exposition, toutes ses autres œuvres libres de droits dans
//     les musées du monde, par l'API ARTEON (direct.js : Chicago, Cleveland, New York,
//     Copenhague, Minneapolis, Europeana, Wikidata).
// Tout s'affiche de soi-même, en entier, sans bouton et sans attendre que l'on descende.
import { choixVue, echapper, lien, lienPage, nombre, plier, t, toutAfficher } from "./commun.js";
import { nomsDesMusees, oeuvresEnDirect } from "./direct.js";
import { ouvrirAuClic } from "./visionneuse.js";
import { jsonGarde } from "./memoire.js";

const zone = document.getElementById("oeuvres-liees");
const oeuvresN = (n) => (n === 1 ? t("1 œuvre", "1 work", "1 obra") : `${nombre(n)} ${t("œuvres", "works", "obras")}`);

/** Une grille qui montre toutes ses œuvres, sans bouton et sans attendre. */
function derouler(section, oeuvres, carte, compte) {
  toutAfficher(section.querySelector(".monde-grille"), oeuvres, carte, (montrees, total) => {
    compte.textContent = montrees < total
      ? `${nombre(montrees)} / ${nombre(total)} ${t("œuvres : affichage en cours…", "works: loading…", "obras: cargando…")}`
      : `${oeuvresN(total)}, ${t("toutes affichées", "all shown", "todas mostradas")}`;
  });
}

/** La clé d'un artiste, comme dans Explorer (explorer.js, cleNom). */
function cleDe(alias) {
  return (nom) => {
    let n = String(nom ?? "").trim();
    const m = /^(.+?)\s*\((.+)\)$/.exec(n);
    if (m) {
      const premier = plier(m[1]).split(/\s+/)[0];
      n = plier(m[2]).includes(premier) ? m[2] : m[1];
    }
    const k = plier(n).replace(/[-‐–'’.,]/g, " ").replace(/\s+/g, " ").trim();
    return alias[k] ?? k;
  };
}

async function ouvrir() {
  const nom = zone.dataset.exposition;
  const artistes = JSON.parse(zone.dataset.artistes || "[]");
  const total = document.getElementById("total-oeuvres");
  const comptes = { visite: Number(zone.dataset.visite || 0), collections: 0, musees: 0 };
  const annoncer = () => {
    if (!total) return;
    const n = comptes.visite + comptes.collections + comptes.musees;
    total.textContent = `${t("Toutes les œuvres", "All the works", "Todas las obras")} : ${nombre(n)}${comptes.enCours ? "…" : ""}`;
  };

  // Mosaïque ou cartes, pour les grilles des collections et des musées (celle de la visite
  // 3D, peu nombreuse et avec ses notices, garde ses cartes).
  zone.prepend(choixVue());

  // 1. Les œuvres liées des collections du monde.
  const liees = await jsonGarde(lien(`monde/donnees/expo-${nom}.json`)).catch(() => []);
  const vues = new Set(liees.map((o) => o[0]));
  comptes.collections = liees.length;
  comptes.enCours = artistes.length > 0;
  annoncer();
  const premiere = zone.querySelector(".monde-liees-collections");
  if (liees.length && premiere) {
    const inconnu = t("Artiste inconnu", "Unknown artist", "Artista desconocido");
    derouler(premiere, liees, ([id, titre, artiste, date, vu]) => `<a class="monde-carte" href="${lienPage(`monde/#/oeuvre/${id}`)}"><img loading="lazy" decoding="async" src="${echapper(vu || lien(`monde/v/${id}.webp`))}" alt=""><b>${echapper(titre)}</b><span>${echapper([artiste === "Artiste inconnu" ? inconnu : artiste, date].filter(Boolean).join(" · "))}</span></a>`,
    premiere.querySelector(".monde-compte"));
  } else if (premiere) {
    premiere.hidden = true;
  }

  // 2. Toutes les autres œuvres de chaque artiste, dans les musées du monde : une section
  // par artiste, demandée à l'API dès l'ouverture de la page.
  const [alias, musees] = await Promise.all([
    jsonGarde(lien("monde/alias.json")).catch(() => ({})),
    nomsDesMusees().catch(() => ({})),
  ]);
  const cle = cleDe(alias);
  let restants = artistes.length;
  const fini = () => { restants -= 1; if (restants <= 0) { comptes.enCours = false; annoncer(); } };
  // Les catalogues sont demandés trois à la fois (fichiers de l'API, ou musées en direct :
  // pas plus, pour ne pas se faire limiter), et affichés dans l'ordre des artistes.
  let actifs = 0;
  const file = [];
  const prendre = () => new Promise((r) => { if (actifs < 3) { actifs += 1; r(); } else file.push(r); });
  const rendre = () => { const r = file.shift(); if (r) r(); else actifs -= 1; };
  const demandes = artistes.map((artiste) => prendre().then(() => oeuvresEnDirect(artiste, cle, alias)).finally(rendre));
  demandes.forEach((d) => d.catch(() => {}));
  for (const [i, artiste] of artistes.entries()) {
    const section = document.createElement("section");
    section.className = "monde-liees-artiste";
    section.innerHTML = `<h2 class="section-title">${echapper(artiste)} · ${t("toutes ses œuvres dans les musées du monde", "all their works in the world's museums", "todas sus obras en los museos del mundo")}</h2>
      <p class="monde-compte">${t("Recherche dans les musées…", "Searching the museums…", "Buscando en los museos…")}</p><div class="monde-grille"></div>`;
    zone.append(section);
    const compte = section.querySelector(".monde-compte");
    // Tout est demandé dès l'ouverture : rien à faire défiler pour tout voir.
    try {
      const r = await demandes[i];
      const oeuvres = (r.oeuvres || []).filter((o) => !vues.has(o.id) && vues.add(o.id));
      if (!oeuvres.length) {
        section.hidden = true;
      } else {
        comptes.musees += oeuvres.length;
        annoncer();
        ouvrirAuClic(section.querySelector(".monde-grille"), oeuvres, musees);
        derouler(section, oeuvres, (o) => `<a class="monde-carte" data-oeuvre="${echapper(o.id)}" href="${echapper(o.lien)}" target="_blank" rel="noopener"><img loading="lazy" decoding="async" src="${echapper(o.vignette)}" alt=""><b>${echapper(o.titre)}</b><span>${echapper([o.date, o.fournisseur && !o.fournisseur.startsWith("http") ? `${o.fournisseur} · ${(musees[o.musee] || "").split(" (")[0]}` : musees[o.musee]].filter(Boolean).join(" · "))}</span></a>`, compte);
      }
    } catch {
      compte.textContent = t("Les musées n'ont pas répondu. Réessayer plus tard.", "The museums did not answer. Try again later.", "Los museos no respondieron. Inténtelo más tarde.");
    }
    fini();
  }
  if (!artistes.length) { comptes.enCours = false; annoncer(); }
}

if (zone) ouvrir();
