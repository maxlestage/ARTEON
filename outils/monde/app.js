// Les collections du monde, sur le site : des dizaines de milliers d'œuvres du domaine
// public (Chicago, Cleveland, le Met, Washington). Les fiches arrivent en morceaux
// (donnees/liste.json pour parcourir et chercher, donnees/fiches-NN.json pour le détail),
// les vignettes viennent du site (v/<œuvre>.jpg), l'image en grand du musée qui la publie.
// Écrit par outils/monde_web.py dans monde/ ; adresses en #/… (une seule page).

const racine = document.getElementById("monde");
const etat = { index: null, liste: null, plies: null, fiches: new Map(), vues: [], montrees: 0 };
const PAR_PAGE = 120;

const echapper = (t) => String(t ?? "").replace(/[&<>"']/g, (c) =>
  ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const plier = (t) => String(t ?? "").replace(/œ/g, "oe").replace(/Œ/g, "oe").replace(/æ/g, "ae")
  .normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
const nombre = (n) => n.toLocaleString("fr-FR");
const rangSiecle = (y) => y <= -100000 ? -999999 : (y < 0 ? -(Math.floor((-y - 1) / 100) + 1) : Math.floor((y - 1) / 100) + 1);

// Une ligne de liste.json : [titre, artiste, date, année, catégorie, musée, ratio].
const T = 0, A = 1, D = 2, Y = 3, C = 4, K = 5;

async function charger() {
  racine.innerHTML = `<p class="monde-chargement">Ouverture des collections…</p>`;
  const [index, liste] = await Promise.all([
    fetch("donnees/index.json").then((r) => r.json()),
    fetch("donnees/liste.json").then((r) => r.json()),
  ]);
  etat.index = index;
  etat.liste = liste;
  etat.ids = index.ids;
  etat.position = new Map(index.ids.map((id, i) => [id, i]));
  window.addEventListener("hashchange", afficher);
  afficher();
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
    if (!etat.plies) etat.plies = L.map((l) => plier(`${l[T]} ${l[A]} ${l[D]} ${l[C]} ${etat.index.musees[l[K]].nom}`));
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
  if (r.type === "oeuvre") return fiche(r.valeur);
  // Les œuvres liées à une exposition : chargées à la première demande.
  if (r.type === "exposition" && !etat.expositions) {
    racine.innerHTML = `<p class="monde-chargement">Ouverture de l'exposition…</p>`;
    fetch("donnees/expositions.json").then((x) => (x.ok ? x.json() : {})).catch(() => ({}))
      .then((e) => { etat.expositions = e; if (route().type === "exposition") afficher(); });
    return;
  }
  const vues = filtrer(r);
  etat.vues = vues;
  etat.montrees = 0;
  const ix = etat.index;
  const titre = {
    musee: ix.musees.find((m) => m.cle === r.valeur)?.nom,
    categorie: r.valeur, siecle: ix.siecles.find((s) => String(s[0]) === r.valeur)?.[1],
    artiste: r.valeur, recherche: r.valeur ? `« ${r.valeur} »` : "",
    exposition: etat.expositions?.[r.valeur] ? `Exposition « ${etat.expositions[r.valeur].nom} »` : "",
  }[r.type] || "Toutes les œuvres";
  racine.innerHTML = `
    <h1 class="monde-titre">Les collections du monde</h1>
    <p class="monde-lead">${nombre(ix.total)} œuvres du domaine public, choisies dans les collections ouvertes
      de ${ix.musees.length} grands musées. Les images en grand viennent des musées eux-mêmes (CC0).</p>
    <div class="monde-musees">${ix.musees.map((m) => `
      <a class="monde-musee${r.type === "musee" && r.valeur === m.cle ? " actif" : ""}" style="--couleur:${m.couleur}" href="#/musee/${m.cle}">
        <strong>${echapper(m.nom)}</strong><span>${echapper(m.ville)} · ${nombre(m.oeuvres)} œuvres</span></a>`).join("")}</div>
    <div class="monde-barre">
      <input type="search" id="monde-q" placeholder="Un titre, un artiste, une technique, une année…" value="${r.type === "recherche" ? echapper(r.valeur) : ""}" aria-label="Chercher">
      <select id="monde-cat" aria-label="Catégorie"><option value="">Toutes les catégories</option>
        ${ix.categories.map(([c, n]) => `<option${r.type === "categorie" && r.valeur === c ? " selected" : ""} value="${echapper(c)}">${echapper(c)} (${nombre(n)})</option>`).join("")}</select>
      <select id="monde-siecle" aria-label="Siècle"><option value="">Tous les siècles</option>
        ${ix.siecles.map(([rang, t, n]) => `<option${r.type === "siecle" && r.valeur === String(rang) ? " selected" : ""} value="${rang}">${echapper(t)} (${nombre(n)})</option>`).join("")}</select>
      <button id="monde-hasard" type="button">🎲 Au hasard</button>
      ${r.type ? `<a class="monde-lien" href="#/">Tout voir</a>` : ""}
    </div>
    <p class="monde-compte"><strong>${echapper(titre)}</strong> · ${vues.length === 1 ? "1 œuvre" : `${nombre(vues.length)} œuvres`}${r.type === "exposition"
      ? ` liées dans les collections du monde, de la plus ancienne à la plus récente · <a href="../${encodeURIComponent(r.valeur)}/">← la visite de l'exposition</a>` : ""}</p>
    <div class="monde-grille" id="monde-grille"></div>
    <button class="monde-plus" id="monde-plus" type="button">Afficher plus</button>`;
  let attente;
  const q = document.getElementById("monde-q");
  q.addEventListener("input", () => {
    clearTimeout(attente);
    attente = setTimeout(() => { location.hash = q.value.trim() ? `#/recherche/${encodeURIComponent(q.value.trim())}` : "#/"; }, 350);
  });
  if (r.type === "recherche") { q.focus(); q.setSelectionRange(q.value.length, q.value.length); }
  document.getElementById("monde-cat").addEventListener("change", (e) => { location.hash = e.target.value ? `#/categorie/${encodeURIComponent(e.target.value)}` : "#/"; });
  document.getElementById("monde-siecle").addEventListener("change", (e) => { location.hash = e.target.value ? `#/siecle/${e.target.value}` : "#/"; });
  document.getElementById("monde-hasard").addEventListener("click", () => {
    location.hash = `#/oeuvre/${etat.ids[Math.floor(Math.random() * etat.ids.length)]}`;
  });
  document.getElementById("monde-plus").addEventListener("click", plus);
  plus();
  const bouton = document.getElementById("monde-plus");
  new IntersectionObserver((e) => { if (e[0].isIntersecting) plus(); }, { rootMargin: "800px" }).observe(bouton);
}

function carte(i) {
  const l = etat.liste[i], id = etat.ids[i];
  const ligne = [l[A], l[D]].filter(Boolean).join(" · ");
  return `<a class="monde-carte" href="#/oeuvre/${id}"><img loading="lazy" decoding="async" src="${l[7] || `v/${id}.jpg`}" alt=""><b>${echapper(l[T])}</b><span>${echapper(ligne)}</span></a>`;
}

function plus() {
  const grille = document.getElementById("monde-grille");
  if (!grille) return;
  const suite = etat.vues.slice(etat.montrees, etat.montrees + PAR_PAGE);
  grille.insertAdjacentHTML("beforeend", suite.map(carte).join(""));
  etat.montrees += suite.length;
  const bouton = document.getElementById("monde-plus");
  if (bouton) bouton.hidden = etat.montrees >= etat.vues.length;
}

async function detail(i) {
  const part = Math.floor(i / etat.index.parFichier);
  if (!etat.fiches.has(part)) {
    const nom = `donnees/fiches-${String(part).padStart(3, "0")}.json`;
    etat.fiches.set(part, fetch(nom).then((r) => r.json()));
  }
  return (await etat.fiches.get(part))[i % etat.index.parFichier];
}

async function fiche(id) {
  const i = etat.position.get(id);
  if (i == null) { racine.innerHTML = `<p class="monde-chargement">Cette œuvre n'est pas dans les collections. <a href="#/">Retour</a></p>`; return; }
  const l = etat.liste[i], m = etat.index.musees[l[K]];
  racine.innerHTML = `<p class="monde-chargement">…</p>`;
  const f = await detail(i);
  if (route().valeur !== id) return;
  const vues = etat.vues.length ? etat.vues : [...etat.liste.keys()];
  const ici = vues.indexOf(i);
  const voisin = (d) => (ici >= 0 && vues[ici + d] != null ? etat.ids[vues[ici + d]] : null);
  const avant = voisin(-1), apres = voisin(1);
  const faits = [["Date", l[D]], ["Technique", f.m], ["Mesures", f.s], ["Catégorie", l[C]], ["Origine", f.o],
    ["Musée", `${m.nom}, ${m.ville}`], ["Où la voir", f.g]].filter(([, v]) => v);
  const memes = l[A] && l[A] !== "Artiste inconnu"
    ? [...etat.liste.keys()].filter((j) => j !== i && etat.liste[j][A] === l[A]).slice(0, 12) : [];
  racine.innerHTML = `
    <a class="monde-retour" href="#/">← Toutes les œuvres</a>
    <div class="monde-fiche">
      <div class="monde-image"><img id="monde-img" src="${l[7] || `v/${id}.jpg`}" alt="${echapper(l[T])}"></div>
      <div>
        <h1>${echapper(l[T])}</h1>
        <p class="monde-artiste"><a href="../explorer/#/artiste/${encodeURIComponent(l[A])}" title="Le catalogue complet de l'artiste">${echapper(l[A])}</a>${f.v ? ` · ${echapper(f.v)}` : ""}</p>
        <dl class="monde-cartel">${faits.map(([k, v]) => `<dt>${k}</dt><dd>${echapper(v)}</dd>`).join("")}</dl>
        ${f.x ? `<div class="monde-texte"><p>${echapper(f.x)}</p>${m.cle === "chicago" ? `<p class="monde-note">Texte : Art Institute of Chicago, CC BY 4.0.</p>` : ""}</div>` : ""}
        <div class="monde-nav">
          ${avant ? `<a href="#/oeuvre/${avant}">‹ Précédente</a>` : ""}${apres ? `<a href="#/oeuvre/${apres}">Suivante ›</a>` : ""}
          ${f.w ? `<a href="${echapper(f.w)}" target="_blank" rel="noopener">La fiche du musée ↗</a>` : ""}
        </div>
        <p class="monde-note">${echapper(m.licence)}</p>
      </div>
    </div>
    ${memes.length ? `<section class="monde-section"><h2>Du même artiste</h2><div class="monde-grille">${memes.map(carte).join("")}</div></section>` : ""}`;
  window.scrollTo(0, 0);
  if (f.u) {
    const grande = new Image();
    grande.onload = () => { const img = document.getElementById("monde-img"); if (img && route().valeur === id) img.src = f.u; };
    grande.src = f.u;
  }
}

charger().catch((e) => {
  racine.innerHTML = `<p class="monde-chargement">Les collections n'ont pas pu s'ouvrir (${echapper(e.message)}).</p>`;
});
