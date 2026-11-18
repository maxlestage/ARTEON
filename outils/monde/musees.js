// Les musées, sur le site : les vrais musées dont ARTEON reproduit les bâtiments, pays par
// pays ; chacun avec son histoire, ce qu'ARTEON lui emprunte et son plan interactif (toucher
// une salle montre ses œuvres) ; et le Grand Musée ARTEON, un musée original en six ailes.

import { annee, carteOeuvre, chargerCatalogue, echapper, lien, nombre, pageExposition, pageGalerie, plier, romain, route, rubrique, t } from "./commun.js";

const racine = document.getElementById("monde");
let C, batiments, pays;

const couleur = (ex) => ex.accent || "#1d3f8f";
const paysDe = (ville) => {
  const sans = String(ville).replace(/\([^)]*\)/g, "");
  const ailleurs = t("Ailleurs", "Elsewhere", "Otros lugares");
  return (sans.split(",").pop() || ailleurs).trim() || ailleurs;
};

function indexer() {
  batiments = [];
  for (const ex of C.exhibitions) {
    if (!ex.museum?.name) continue;
    let b = batiments.find((x) => x.museum.name === ex.museum.name);
    if (!b) batiments.push(b = { museum: ex.museum, expos: [] });
    b.expos.push(ex);
  }
  const groupes = new Map();
  for (const b of batiments) {
    const p = paysDe(b.museum.city);
    if (!groupes.has(p)) groupes.set(p, []);
    groupes.get(p).push(b);
  }
  pays = [...groupes].sort((a, b) => b[1].length - a[1].length || plier(a[0]).localeCompare(plier(b[0])));
}

/** Le plan d'un bâtiment, en SVG : salles, portes, tableaux, socles, numéros, entrée. */
function plan(ex, choisie, petit) {
  const salles = ex.rooms.filter((r) => r.width);
  if (!salles.length) return "";
  let x0 = Infinity, z0 = Infinity, x1 = -Infinity, z1 = -Infinity;
  for (const r of salles) {
    x0 = Math.min(x0, r.x - r.width / 2); x1 = Math.max(x1, r.x + r.width / 2);
    z0 = Math.min(z0, r.z - r.depth / 2); z1 = Math.max(z1, r.z + r.depth / 2);
  }
  const pad = 3, ac = couleur(ex);
  let svg = `<svg class="monde-plan${petit ? " petit" : ""}" viewBox="${x0 - pad} ${z0 - pad} ${x1 - x0 + 2 * pad} ${z1 - z0 + 2 * pad}" role="img" aria-label="${t("Le plan du bâtiment", "Floor plan of the building", "El plano del edificio")}">`;
  ex.rooms.forEach((r, i) => {
    if (!r.width) return;
    const fill = r.corridor ? "#5c544d" : (r.walls || "#ecebe4");
    svg += `<rect data-salle="${i}" x="${r.x - r.width / 2}" y="${r.z - r.depth / 2}" width="${r.width}" height="${r.depth}" fill="${fill}" stroke="#33302d" stroke-width="0.2"/>`;
    if (i === choisie) svg += `<rect x="${r.x - r.width / 2}" y="${r.z - r.depth / 2}" width="${r.width}" height="${r.depth}" fill="${ac}" fill-opacity=".22" stroke="${ac}" stroke-width="0.5" pointer-events="none"/>`;
  });
  for (const r of ex.rooms) for (const d of r.doors || []) {
    svg += `<line x1="${d[0]}" y1="${d[1]}" x2="${d[2]}" y2="${d[3]}" stroke="#f7f5ee" stroke-width="0.5" pointer-events="none"/>`;
  }
  for (const w of ex.works) {
    const s = w.spot;
    if (!s) continue;
    svg += s.length === 4
      ? `<line x1="${s[0]}" y1="${s[1]}" x2="${s[2]}" y2="${s[3]}" stroke="${ac}" stroke-width="0.3" stroke-linecap="round" pointer-events="none"/>`
      : `<circle cx="${s[0]}" cy="${s[1]}" r="${s[2]}" fill="${ac}" fill-opacity=".85" stroke="#33302d" stroke-width="0.08" pointer-events="none"/>`;
  }
  if (!petit) for (const r of ex.rooms) {
    if (r.number == null || !r.width) continue;
    const taille = Math.min(Math.max(Math.min(r.width, r.depth) * 0.4, 1.2), 3.2);
    svg += `<text x="${r.x}" y="${r.z + taille * 0.35}" font-size="${taille}" text-anchor="middle" fill="#33302d" fill-opacity=".75" font-weight="700" pointer-events="none">${r.number}</text>`;
  }
  const e = ex.entrance;
  if (!petit && e?.length === 4) {
    svg += `<text x="${e[0] - e[2] * 1.8}" y="${e[1] - e[3] * 1.8 + 0.5}" font-size="1.3" text-anchor="middle" fill="#33302d" pointer-events="none">▲ ${t("Entrée", "Entrance", "Entrada")}</text>`;
  }
  return svg + "</svg>";
}

function accueil() {
  const total = batiments.length;
  racine.innerHTML = `
    <h1 class="monde-titre">${t("Les musées", "The museums", "Los museos")}</h1>
    <p class="monde-lead">${t(
      `Chaque exposition d'ARTEON est installée dans la reproduction d'un vrai musée : sa façade,
      ses enfilades, ses verrières, sa lumière. ${nombre(total)} musées dans ${nombre(pays.length)} pays, et un musée original.`,
      `Every ARTEON exhibition is set in a reproduction of a real museum: its façade,
      its suites of rooms, its glass roofs, its light. ${nombre(total)} museums in ${nombre(pays.length)} ${pays.length === 1 ? "country" : "countries"}, and one original museum.`,
      `Cada exposición de ARTEON se instala en la reproducción de un museo real: su fachada,
      sus galerías en enfilada, sus cristaleras, su luz. ${nombre(total)} museos en ${nombre(pays.length)} ${pays.length === 1 ? "país" : "países"}, y un museo original.`)}</p>
    <a class="monde-accueil" href="#/grand-musee"><div><strong>${t("Le Grand Musée ARTEON", "The ARTEON Grand Museum", "El Gran Museo ARTEON")}</strong>
      <span>${t(
        `Un musée original, qui n'existe que dans ARTEON : ${nombre(C.exhibitions.length)} pavillons en six ailes, de la Terre aux avant-gardes.`,
        `An original museum that exists only in ARTEON: ${nombre(C.exhibitions.length)} pavilions in six wings, from the Earth to the avant-gardes.`,
        `Un museo original que solo existe en ARTEON: ${nombre(C.exhibitions.length)} pabellones en seis alas, de la Tierra a las vanguardias.`)}</span></div></a>
    ${pays.map(([p, bs]) => `
      <section class="monde-section"><h2>${echapper(p)}</h2>
        <div class="monde-grille batiments">${bs.map((b) => {
          const ex = b.expos[0];
          return `<a class="monde-carte batiment" href="#/musee/${encodeURIComponent(b.museum.name)}" style="--couleur:${couleur(ex)}">
            <div class="batiment-haut"><img loading="lazy" src="${lien(ex.cover || ex.works[0]?.image)}" alt="">${plan(ex, -1, true)}</div>
            <b>${echapper(b.museum.name)}</b><span>${echapper(b.museum.city)}</span>
            <span>${echapper([b.museum.architect, b.museum.built].filter(Boolean).join(" · "))}</span></a>`;
        }).join("")}</div></section>`).join("")}`;
}

function musee(nom) {
  const b = batiments.find((x) => x.museum.name === nom);
  if (!b) { racine.innerHTML = `<p class="monde-chargement">${t("Musée introuvable.", "Museum not found.", "Museo no encontrado.")} <a href="#/">${t("Tous les musées", "All museums", "Todos los museos")}</a></p>`; return; }
  const m = b.museum, ac = couleur(b.expos[0]);
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← ${t("Tous les musées", "All museums", "Todos los museos")}</a>
    <div class="monde-bandeau" style="--couleur:${ac}"><img src="${lien(b.expos[0].cover || b.expos[0].works[0]?.image)}" alt="">
      <div><h1>${echapper(m.name)}</h1><p>${echapper(m.city)}</p></div></div>
    <dl class="monde-cartel">${[[t("Architecte", "Architect", "Arquitecto"), m.architect], [t("Construction", "Built", "Construcción"), m.built], [t("Pays", "Country", "País"), paysDe(m.city)]]
      .filter(([, v]) => v).map(([k, v]) => `<dt>${k}</dt><dd>${echapper(v)}</dd>`).join("")}</dl>
    ${m.presentation || m.here ? `<div class="folds">${m.presentation ? rubrique(t("Le vrai musée", "The real museum", "El museo real"), `<p>${echapper(m.presentation)}</p>`, true) : ""}${m.here ? rubrique(t("Ce qu'ARTEON lui emprunte", "What ARTEON borrows from it", "Lo que ARTEON toma de él"), `<p>${echapper(m.here)}</p>`) : ""}</div>` : ""}
    ${b.expos.map((ex, n) => `
      <section class="monde-section" data-expo="${n}">
        <h2>${b.expos.length > 1 ? echapper(ex.short_name || ex.title) : t("Le plan", "Floor plan", "El plano")}</h2>
        <div class="monde-plan-cadre">${plan(ex, -1)}</div>
        <p class="monde-note">${t("Toucher une salle pour voir ses œuvres.", "Tap a room to see its works.", "Toque una sala para ver sus obras.")}</p>
        <div class="monde-salle"></div>
        <div class="monde-nav"><a href="${pageGalerie(ex)}">${t("Entrer en 3D", "Enter in 3D", "Entrar en 3D")}</a><a href="${pageExposition(ex)}">${t("L'exposition", "The exhibition", "La exposición")}</a></div>
      </section>`).join("")}`;
  racine.querySelectorAll("[data-expo]").forEach((section) => {
    const ex = b.expos[Number(section.dataset.expo)];
    const cadre = section.querySelector(".monde-plan-cadre");
    const salle = section.querySelector(".monde-salle");
    cadre.addEventListener("click", (e) => {
      const i = e.target.closest("rect[data-salle]")?.dataset.salle;
      if (i == null) return;
      const r = ex.rooms[Number(i)];
      cadre.innerHTML = plan(ex, Number(i));
      const oeuvres = ex.works.filter((w) => w.room === Number(i));
      salle.innerHTML = `<h3>${r.number ? `${t("Salle", "Room", "Sala")} ${r.number} · ` : ""}${echapper(r.name || (r.corridor ? t("Un couloir", "A corridor", "Un pasillo") : t("Une salle", "A room", "Una sala")))}</h3>
        ${r.theme ? `<p class="monde-note">${echapper(r.theme)}</p>` : ""}
        ${oeuvres.length ? `<div class="monde-grille">${oeuvres.map((w) => carteOeuvre(ex, w)).join("")}</div>
          <div class="monde-nav"><a href="${pageGalerie(ex, oeuvres[0])}">${t("Entrer dans cette salle en 3D", "Enter this room in 3D", "Entrar en esta sala en 3D")}</a></div>`
          : `<p class="monde-note">${r.corridor ? t("Un passage entre les salles.", "A passage between the rooms.", "Un paso entre las salas.") : t("Pas d'œuvre dans cette salle.", "No works in this room.", "No hay obras en esta sala.")}</p>`}`;
    });
  });
  window.scrollTo(0, 0);
}

function grandMusee() {
  const ailes = [
    [t("L'aile des origines", "The Wing of Origins", "El ala de los orígenes"), t("De la Terre aux premiers empires", "From the Earth to the first empires", "De la Tierra a los primeros imperios"), 500],
    [t("L'aile des cathédrales", "The Wing of Cathedrals", "El ala de las catedrales"), t("Byzance, l'Islam, le Moyen Âge, l'Asie", "Byzantium, Islam, the Middle Ages, Asia", "Bizancio, el islam, la Edad Media, Asia"), 1400],
    [t("L'aile des palais", "The Wing of Palaces", "El ala de los palacios"), t("Renaissance et Baroque", "Renaissance and Baroque", "Renacimiento y Barroco"), 1700],
    [t("L'aile des salons", "The Wing of Salons", "El ala de los salones"), t("Des Lumières au romantisme", "From the Enlightenment to Romanticism", "De la Ilustración al Romanticismo"), 1850],
    [t("L'aile de la lumière", "The Wing of Light", "El ala de la luz"), t("La seconde moitié du XIXᵉ siècle", "The second half of the 19th century", "La segunda mitad del siglo XIX"), 1900],
    [t("L'aile moderne", "The Modern Wing", "El ala moderna"), t("Le XXᵉ siècle et aujourd'hui", "The 20th century and today", "El siglo XX y la actualidad"), Infinity],
  ].map(([titre, s, jusque]) => ({ t: titre, s, jusque, expos: [] }));
  const datees = C.exhibitions.map((ex) => {
    const ans = ex.works.map((w) => annee(w.year)).filter((y) => y != null).sort((a, b) => a - b);
    return { ex, y: ans.length ? ans[Math.floor(ans.length / 2)] : 1900 };
  }).sort((a, b) => a.y - b.y);
  for (const { ex, y } of datees) (ailes.find((a) => y < a.jusque) || ailes.at(-1)).expos.push(ex);
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← ${t("Tous les musées", "All museums", "Todos los museos")}</a>
    <div class="grand-hall"><h1 class="monde-titre">${t("Le Grand Musée ARTEON", "The ARTEON Grand Museum", "El Gran Museo ARTEON")}</h1>
      <p class="monde-lead">${t(
        `Un musée original, qui n'existe que dans ARTEON : ${nombre(C.exhibitions.length)} pavillons, chacun dans la
        reproduction d'un vrai musée, réunis en ${nombre(ailes.filter((a) => a.expos.length).length)} ailes. On entre par le hall, on suit le temps.`,
        `An original museum that exists only in ARTEON: ${nombre(C.exhibitions.length)} pavilions, each in a
        reproduction of a real museum, brought together in ${nombre(ailes.filter((a) => a.expos.length).length)} wings. You enter through the hall and follow the course of time.`,
        `Un museo original que solo existe en ARTEON: ${nombre(C.exhibitions.length)} pabellones, cada uno en la
        reproducción de un museo real, reunidos en ${nombre(ailes.filter((a) => a.expos.length).length)} alas. Se entra por el vestíbulo y se sigue el curso del tiempo.`)}</p>
      <span class="grand-porte">${t("Hall d'entrée", "Entrance hall", "Vestíbulo de entrada")}</span></div>
    ${ailes.filter((a) => a.expos.length).map((a, n) => `
      <div class="grand-couloir"></div>
      <section class="grand-aile"><p class="monde-note">${t("AILE", "WING", "ALA")} ${romain(n + 1)}</p><h2>${echapper(a.t)}</h2><p class="monde-note">${echapper(a.s)}</p>
        <div class="grand-pavillons">${a.expos.map((ex) => `
          <a class="grand-pavillon" style="--couleur:${couleur(ex)}" href="${ex.museum ? `#/musee/${encodeURIComponent(ex.museum.name)}` : pageExposition(ex)}">
            <img loading="lazy" src="${lien(ex.cover || ex.works[0]?.image)}" alt=""><b>${echapper(ex.short_name || ex.title)}</b>
            <span>${echapper(ex.museum?.name || ex.movement)}</span></a>`).join("")}</div></section>`).join("")}
    <div class="grand-couloir"></div><p class="monde-note" style="text-align:center">${t("Sortie · Jardin des sculptures", "Exit · Sculpture garden", "Salida · Jardín de esculturas")}</p>`;
  window.scrollTo(0, 0);
}

function afficher() {
  const r = route();
  if (r.type === "musee") return musee(r.valeur);
  if (r.type === "grand-musee") return grandMusee();
  accueil();
}

racine.innerHTML = `<p class="monde-chargement">${t("Ouverture des musées…", "Opening the museums…", "Abriendo los museos…")}</p>`;
chargerCatalogue().then((c) => {
  C = c;
  indexer();
  window.addEventListener("hashchange", afficher);
  afficher();
}).catch((e) => { racine.innerHTML = `<p class="monde-chargement">${t("Les musées n'ont pas pu s'ouvrir", "The museums could not be opened", "No se han podido abrir los museos")} (${echapper(e.message)}).</p>`; });
