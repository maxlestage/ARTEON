// La visionneuse : une œuvre des musées du monde ouverte en grand, sans quitter ARTEON.
// Un clic sur une carte « en direct » (page d'artiste de l'Explorer, œuvres liées d'une
// exposition) l'ouvre : la grande image, sa fiche (artiste, date, technique, collection,
// musée), le lien vers la fiche du musée, et la précédente / la suivante (flèches,
// boutons ou glissement du doigt). Retour du navigateur, Échap ou ✕ : on retrouve la liste,
// là où on était. Sans JavaScript, la carte mène toujours chez le musée.

import { echapper, t } from "./commun.js";

let boite = null;
let etat = null;

function construire() {
  boite = document.createElement("div");
  boite.className = "visionneuse";
  boite.hidden = true;
  boite.setAttribute("role", "dialog");
  boite.setAttribute("aria-modal", "true");
  boite.innerHTML = `
    <div class="visionneuse-barre">
      <span class="visionneuse-compte"></span>
      <button type="button" class="visionneuse-fermer" aria-label="${t("Fermer", "Close", "Cerrar")}">✕</button>
    </div>
    <div class="visionneuse-scene">
      <button type="button" class="visionneuse-fleche" data-pas="-1" aria-label="${t("Œuvre précédente", "Previous work", "Obra anterior")}">‹</button>
      <figure class="visionneuse-image"><img alt=""></figure>
      <button type="button" class="visionneuse-fleche" data-pas="1" aria-label="${t("Œuvre suivante", "Next work", "Obra siguiente")}">›</button>
    </div>
    <div class="visionneuse-fiche"></div>`;
  document.body.append(boite);
  boite.querySelector(".visionneuse-fermer").addEventListener("click", () => history.back());
  for (const b of boite.querySelectorAll(".visionneuse-fleche")) {
    b.addEventListener("click", () => montrer(etat.i + Number(b.dataset.pas)));
  }
  document.addEventListener("keydown", (e) => {
    if (boite.hidden) return;
    if (e.key === "Escape") history.back();
    else if (e.key === "ArrowLeft") montrer(etat.i - 1);
    else if (e.key === "ArrowRight") montrer(etat.i + 1);
  });
  // Le retour du navigateur ferme la visionneuse (elle a ajouté une entrée à l'historique).
  window.addEventListener("popstate", () => { if (!boite.hidden) fermer(); });
  // Glisser le doigt vers la gauche ou la droite : la suivante ou la précédente.
  let x0 = null, y0 = null;
  const scene = boite.querySelector(".visionneuse-scene");
  scene.addEventListener("touchstart", (e) => { x0 = e.touches[0].clientX; y0 = e.touches[0].clientY; }, { passive: true });
  scene.addEventListener("touchend", (e) => {
    if (x0 == null) return;
    const dx = e.changedTouches[0].clientX - x0, dy = e.changedTouches[0].clientY - y0;
    x0 = null;
    if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) montrer(etat.i + (dx < 0 ? 1 : -1));
  });
}

function fermer() {
  boite.hidden = true;
  document.documentElement.classList.remove("visionneuse-ouverte");
  etat?.depuis?.focus?.({ preventScroll: true });
  etat = null;
}

function montrer(i) {
  const { oeuvres, musees } = etat;
  if (i < 0 || i >= oeuvres.length) return;
  etat.i = i;
  const o = oeuvres[i];
  boite.querySelector(".visionneuse-compte").textContent = `${(i + 1).toLocaleString()} / ${oeuvres.length.toLocaleString()}`;
  const [avant, apres] = boite.querySelectorAll(".visionneuse-fleche");
  avant.disabled = i === 0;
  apres.disabled = i === oeuvres.length - 1;
  // La vignette tout de suite, la grande image dès qu'elle est arrivée.
  const img = boite.querySelector(".visionneuse-image img");
  img.src = o.vignette || o.image || "";
  img.alt = o.titre || "";
  if (o.image && o.image !== o.vignette) {
    const grande = new Image();
    grande.onload = () => { if (etat && etat.i === i) img.src = o.image; };
    grande.src = o.image;
  }
  const musee = (musees[o.musee] || o.musee || "").split(" (")[0];
  const faits = [
    [t("Artiste", "Artist", "Artista"), o.artiste],
    [t("Date", "Date", "Fecha"), o.date],
    [t("Technique", "Medium", "Técnica"), o.technique],
    [t("Collection", "Collection", "Colección"), o.fournisseur],
    [t("Source", "Source", "Fuente"), musee],
  ].filter(([, v]) => v);
  boite.querySelector(".visionneuse-fiche").innerHTML = `
    <h2>${echapper(o.titre || t("Sans titre", "Untitled", "Sin título"))}</h2>
    <dl class="monde-cartel">${faits.map(([k, v]) => `<dt>${k}</dt><dd>${echapper(v)}</dd>`).join("")}</dl>
    ${o.lien ? `<p><a class="visionneuse-musee" href="${echapper(o.lien)}" target="_blank" rel="noopener">${t("La fiche du musée", "On the museum's website", "La ficha del museo")} ↗</a></p>` : ""}
    <p class="monde-note">${t("Domaine public · image et fiche chez le musée", "Public domain · image and record hosted by the museum", "Dominio público · imagen y ficha en el museo")}</p>`;
  boite.querySelector(".visionneuse-fiche").scrollTop = 0;
}

/** Ouvre la visionneuse sur `oeuvres[i]` (œuvres de l'API : titre, vignette, image, lien…). */
export function ouvrir(oeuvres, i, musees = {}) {
  if (!boite) construire();
  etat = { oeuvres, i, musees, depuis: document.activeElement };
  boite.hidden = false;
  document.documentElement.classList.add("visionneuse-ouverte");
  history.pushState({ visionneuse: true }, "");
  montrer(i);
  boite.querySelector(".visionneuse-fermer").focus({ preventScroll: true });
}

/** Un clic sur une carte `[data-oeuvre]` de `grille` ouvre la visionneuse sur cette œuvre ;
 *  Cmd/Ctrl-clic ou clic du milieu ouvrent toujours la fiche du musée dans un onglet. */
export function ouvrirAuClic(grille, oeuvres, musees = {}) {
  const position = new Map(oeuvres.map((o, i) => [String(o.id), i]));
  grille.addEventListener("click", (e) => {
    const carte = e.target.closest("a[data-oeuvre]");
    if (!carte || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const i = position.get(carte.dataset.oeuvre);
    if (i == null) return;
    e.preventDefault();
    ouvrir(oeuvres, i, musees);
  });
}
