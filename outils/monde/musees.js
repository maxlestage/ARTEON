// Les musées, sur le site : les vrais musées dont ARTEON reproduit les bâtiments, pays par
// pays ; chacun avec son histoire, ce qu'ARTEON lui emprunte et son plan interactif (toucher
// une salle montre ses œuvres) ; et le Grand Musée ARTEON, un musée original en six ailes.

import { annee, carteOeuvre, chargerCatalogue, echapper, lien, pageExposition, pageGalerie, plier, romain, route } from "./commun.js";

const racine = document.getElementById("monde");
let C, batiments, pays;

const couleur = (ex) => ex.accent || "#1d3f8f";
const paysDe = (ville) => {
  const sans = String(ville).replace(/\([^)]*\)/g, "");
  return (sans.split(",").pop() || "Ailleurs").trim() || "Ailleurs";
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
  let svg = `<svg class="monde-plan${petit ? " petit" : ""}" viewBox="${x0 - pad} ${z0 - pad} ${x1 - x0 + 2 * pad} ${z1 - z0 + 2 * pad}" role="img" aria-label="Le plan du bâtiment">`;
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
    const t = Math.min(Math.max(Math.min(r.width, r.depth) * 0.4, 1.2), 3.2);
    svg += `<text x="${r.x}" y="${r.z + t * 0.35}" font-size="${t}" text-anchor="middle" fill="#33302d" fill-opacity=".75" font-weight="700" pointer-events="none">${r.number}</text>`;
  }
  const e = ex.entrance;
  if (!petit && e?.length === 4) {
    svg += `<text x="${e[0] - e[2] * 1.8}" y="${e[1] - e[3] * 1.8 + 0.5}" font-size="1.3" text-anchor="middle" fill="#33302d" pointer-events="none">▲ Entrée</text>`;
  }
  return svg + "</svg>";
}

function accueil() {
  const total = batiments.length;
  racine.innerHTML = `
    <h1 class="monde-titre">Les musées</h1>
    <p class="monde-lead">Chaque exposition d'ARTEON est installée dans la reproduction d'un vrai musée : sa façade,
      ses enfilades, ses verrières, sa lumière. ${total} musées dans ${pays.length} pays, et un musée original.</p>
    <a class="monde-accueil" href="#/grand-musee"><div><strong>Le Grand Musée ARTEON</strong>
      <span>Un musée original, qui n'existe que dans ARTEON : ${C.exhibitions.length} pavillons en six ailes, de la Terre aux avant-gardes.</span></div></a>
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
  if (!b) { racine.innerHTML = `<p class="monde-chargement">Musée introuvable. <a href="#/">Tous les musées</a></p>`; return; }
  const m = b.museum, ac = couleur(b.expos[0]);
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← Tous les musées</a>
    <div class="monde-bandeau" style="--couleur:${ac}"><img src="${lien(b.expos[0].cover || b.expos[0].works[0]?.image)}" alt="">
      <div><h1>${echapper(m.name)}</h1><p>${echapper(m.city)}</p></div></div>
    <dl class="monde-cartel">${[["Architecte", m.architect], ["Construction", m.built], ["Pays", paysDe(m.city)]]
      .filter(([, v]) => v).map(([k, v]) => `<dt>${k}</dt><dd>${echapper(v)}</dd>`).join("")}</dl>
    ${m.presentation ? `<section class="monde-section"><h2 style="color:${ac}">Le vrai musée</h2><p class="monde-texte">${echapper(m.presentation)}</p></section>` : ""}
    ${m.here ? `<section class="monde-section"><h2 style="color:${ac}">Ce qu'ARTEON lui emprunte</h2><p class="monde-texte">${echapper(m.here)}</p></section>` : ""}
    ${b.expos.map((ex, n) => `
      <section class="monde-section" data-expo="${n}">
        <h2>${b.expos.length > 1 ? echapper(ex.short_name || ex.title) : "Le plan"}</h2>
        <div class="monde-plan-cadre">${plan(ex, -1)}</div>
        <p class="monde-note">Toucher une salle pour voir ses œuvres.</p>
        <div class="monde-salle"></div>
        <div class="monde-nav"><a href="${pageGalerie(ex)}">Entrer en 3D</a><a href="${pageExposition(ex)}">L'exposition</a></div>
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
      salle.innerHTML = `<h3>${r.number ? `Salle ${r.number} · ` : ""}${echapper(r.name || (r.corridor ? "Un couloir" : "Une salle"))}</h3>
        ${r.theme ? `<p class="monde-note">${echapper(r.theme)}</p>` : ""}
        ${oeuvres.length ? `<div class="monde-grille">${oeuvres.map((w) => carteOeuvre(ex, w)).join("")}</div>
          <div class="monde-nav"><a href="${pageGalerie(ex, oeuvres[0])}">Entrer dans cette salle en 3D</a></div>`
          : `<p class="monde-note">${r.corridor ? "Un passage entre les salles." : "Pas d'œuvre dans cette salle."}</p>`}`;
    });
  });
  window.scrollTo(0, 0);
}

function grandMusee() {
  const ailes = [
    ["L'aile des origines", "De la Terre aux premiers empires", 500],
    ["L'aile des cathédrales", "Byzance, l'Islam, le Moyen Âge, l'Asie", 1400],
    ["L'aile des palais", "Renaissance et Baroque", 1700],
    ["L'aile des salons", "Des Lumières au romantisme", 1850],
    ["L'aile de la lumière", "La seconde moitié du XIXᵉ siècle", 1900],
    ["L'aile moderne", "Le XXᵉ siècle et aujourd'hui", Infinity],
  ].map(([t, s, jusque]) => ({ t, s, jusque, expos: [] }));
  const datees = C.exhibitions.map((ex) => {
    const ans = ex.works.map((w) => annee(w.year)).filter((y) => y != null).sort((a, b) => a - b);
    return { ex, y: ans.length ? ans[Math.floor(ans.length / 2)] : 1900 };
  }).sort((a, b) => a.y - b.y);
  for (const { ex, y } of datees) (ailes.find((a) => y < a.jusque) || ailes.at(-1)).expos.push(ex);
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← Tous les musées</a>
    <div class="grand-hall"><h1 class="monde-titre">Le Grand Musée ARTEON</h1>
      <p class="monde-lead">Un musée original, qui n'existe que dans ARTEON : ${C.exhibitions.length} pavillons, chacun dans la
        reproduction d'un vrai musée, réunis en ${ailes.filter((a) => a.expos.length).length} ailes. On entre par le hall, on suit le temps.</p>
      <span class="grand-porte">Hall d'entrée</span></div>
    ${ailes.filter((a) => a.expos.length).map((a, n) => `
      <div class="grand-couloir"></div>
      <section class="grand-aile"><p class="monde-note">AILE ${romain(n + 1)}</p><h2>${echapper(a.t)}</h2><p class="monde-note">${echapper(a.s)}</p>
        <div class="grand-pavillons">${a.expos.map((ex) => `
          <a class="grand-pavillon" style="--couleur:${couleur(ex)}" href="${ex.museum ? `#/musee/${encodeURIComponent(ex.museum.name)}` : pageExposition(ex)}">
            <img loading="lazy" src="${lien(ex.cover || ex.works[0]?.image)}" alt=""><b>${echapper(ex.short_name || ex.title)}</b>
            <span>${echapper(ex.museum?.name || ex.movement)}</span></a>`).join("")}</div></section>`).join("")}
    <div class="grand-couloir"></div><p class="monde-note" style="text-align:center">Sortie · Jardin des sculptures</p>`;
  window.scrollTo(0, 0);
}

function afficher() {
  const r = route();
  if (r.type === "musee") return musee(r.valeur);
  if (r.type === "grand-musee") return grandMusee();
  accueil();
}

racine.innerHTML = `<p class="monde-chargement">Ouverture des musées…</p>`;
chargerCatalogue().then((c) => {
  C = c;
  indexer();
  window.addEventListener("hashchange", afficher);
  afficher();
}).catch((e) => { racine.innerHTML = `<p class="monde-chargement">Les musées n'ont pas pu s'ouvrir (${echapper(e.message)}).</p>`; });
