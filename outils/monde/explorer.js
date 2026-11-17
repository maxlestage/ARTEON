// Explorer le musée autrement, sur le site : les artistes de A à Z (et leurs œuvres dans
// les collections du monde), la chronologie siècle par siècle, les thèmes, et le jeu
// « Qui l'a fait ? » (le record est gardé dans le navigateur).

import { annee, carteOeuvre, chargerCatalogue, echapper, lien, nombre, plier, rangSiecle, route, titreSiecle, toutes } from "./commun.js";

const racine = document.getElementById("monde");
let C, oeuvres, artistes, siecles, themes, mondeIndex;

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

const menu = (actif) => `
  <h1 class="monde-titre">Explorer</h1>
  <div class="monde-onglets">${[["", "Les artistes"], ["chronologie", "La chronologie"], ["themes", "Les thèmes"], ["jeu", "Qui l'a fait ?"]]
    .map(([k, t]) => `<a class="${actif === k ? "actif" : ""}" href="#/${k}">${t}</a>`).join("")}</div>`;

function listeArtistes() {
  racine.innerHTML = menu("") + `
    <p class="monde-lead">${nombre(artistes.length)} artistes, ateliers et cultures, de A à Z.</p>
    <div class="monde-barre"><input type="search" id="filtre" placeholder="Un nom, un style…" aria-label="Chercher un artiste"></div>
    <div id="artistes"></div>`;
  const zone = document.getElementById("artistes");
  const dessiner = (q) => {
    const f = plier(q.trim());
    const liste = f ? artistes.filter((a) => plier(`${a.nom} ${a.style} ${a.vie}`).includes(f)) : artistes;
    let lettre = "", html = "";
    for (const a of liste) {
      const l = (plier(a.nom)[0] || "#").toUpperCase();
      if (l !== lettre) { html += `${lettre ? "</div>" : ""}<h2 class="monde-lettre">${/[A-Z]/.test(l) ? l : "#"}</h2><div class="monde-artistes">`; lettre = l; }
      html += `<a class="monde-artiste-ligne" href="#/artiste/${encodeURIComponent(a.nom)}"><img loading="lazy" src="${lien(a.oeuvres[0].w.image)}" alt="">
        <span><b>${echapper(a.nom)}</b><small>${echapper([a.vie, a.style].filter(Boolean).join(" · "))}</small></span><em>${a.oeuvres.length}</em></a>`;
    }
    zone.innerHTML = html + (lettre ? "</div>" : `<p class="monde-chargement">Personne ne s'appelle ainsi.</p>`);
  };
  document.getElementById("filtre").addEventListener("input", (e) => dessiner(e.target.value));
  dessiner("");
}

async function artiste(nom) {
  const a = artistes.find((x) => x.nom === nom);
  if (!a) { racine.innerHTML = `<p class="monde-chargement">Artiste introuvable. <a href="#/">Les artistes</a></p>`; return; }
  mondeIndex ??= fetch(lien("monde/donnees/liste.json")).then((r) => (r.ok ? r.json() : [])).catch(() => []);
  racine.innerHTML = `<a class="monde-retour" href="#/">← Les artistes</a>
    <h1 class="monde-titre">${echapper(a.nom)}</h1>
    <p class="monde-artiste">${echapper([a.vie, a.style].filter(Boolean).join(" · "))}</p>
    ${a.bio ? `<p class="monde-texte">${echapper(a.bio)}</p>` : ""}
    <section class="monde-section"><h2>${a.oeuvres.length === 1 ? "Son œuvre au musée" : `Ses ${a.oeuvres.length} œuvres au musée`}</h2>
      <div class="monde-grille">${a.oeuvres.map(({ ex, w }) => carteOeuvre(ex, w, ex.short_name || ex.title)).join("")}</div></section>
    <section class="monde-section" id="dans-le-monde"></section>`;
  window.scrollTo(0, 0);
  const liste = await mondeIndex;
  const n = liste.filter((l) => l[1] === a.nom).length;
  const zone = document.getElementById("dans-le-monde");
  if (n && zone) zone.innerHTML = `<h2>Dans les collections du monde</h2>
    <a class="monde-lien" href="${lien(`monde/#/artiste/${encodeURIComponent(a.nom)}`)}">${n === 1 ? "1 œuvre" : `${nombre(n)} œuvres`} de ${echapper(a.nom)} →</a>`;
}

function chronologie() {
  racine.innerHTML = menu("chronologie") + `
    <p class="monde-lead">Toutes les œuvres du musée, de la plus ancienne à la plus récente, siècle par siècle.</p>
    <div class="monde-barre"><select id="aller" aria-label="Aller au siècle"><option value="">Aller au siècle…</option>
      ${siecles.map((s) => `<option value="s${s.r}">${echapper(s.titre)} (${s.oeuvres.length})</option>`).join("")}</select></div>
    ${siecles.map((s) => `<section class="monde-section" id="s${s.r}"><h2>${echapper(s.titre)} <small class="monde-note">${s.oeuvres.length} œuvres ·
        <a href="${lien(`monde/#/siecle/${s.r}`)}">dans les collections du monde</a></small></h2>
      <div class="monde-rangee">${s.oeuvres.map(({ ex, w }) => carteOeuvre(ex, w)).join("")}</div></section>`).join("")}`;
  document.getElementById("aller").addEventListener("change", (e) => document.getElementById(e.target.value)?.scrollIntoView({ behavior: "smooth" }));
}

function listeThemes(choisi) {
  const t = choisi && themes.find((x) => x.titre === choisi);
  racine.innerHTML = menu("themes") + (t ? `
    <a class="monde-retour" href="#/themes">← Tous les thèmes</a><h2>${echapper(t.titre)} <small class="monde-note">${t.oeuvres.length} œuvres</small></h2>
    <div class="monde-grille">${t.oeuvres.map(({ ex, w }) => carteOeuvre(ex, w)).join("")}</div>` : `
    <p class="monde-lead">${themes.length} thèmes : la nuit, la mer, l'amour, la mort, le portrait…</p>
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
    <p class="monde-lead">🔥 Série : <b>${serie}</b> · 🏆 Record : <b>${record()}</b></p>
    <div class="jeu-image"><img src="${lien(w.image)}" alt="L'œuvre à reconnaître"></div>
    <h2>Qui a fait cette œuvre ?</h2>
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
      <div class="monde-nav"><a href="${lien(`${ex.slug}/oeuvres/${w.slug}/`)}">Voir la notice</a><a href="#/jeu" id="jeu-encore">${bon ? "Suivante ›" : "Rejouer"}</a></div>`;
    document.getElementById("jeu-encore").addEventListener("click", (e) => { e.preventDefault(); jeu(); });
  }));
}

function afficher() {
  const r = route();
  if (r.type === "artiste") return artiste(r.valeur);
  if (r.type === "chronologie") return chronologie();
  if (r.type === "themes") return listeThemes(r.valeur);
  if (r.type === "jeu") return jeu();
  listeArtistes();
}

racine.innerHTML = `<p class="monde-chargement">Ouverture du musée…</p>`;
chargerCatalogue().then((c) => {
  C = c;
  indexer();
  window.addEventListener("hashchange", afficher);
  afficher();
}).catch((e) => { racine.innerHTML = `<p class="monde-chargement">Le musée n'a pas pu s'ouvrir (${echapper(e.message)}).</p>`; });
