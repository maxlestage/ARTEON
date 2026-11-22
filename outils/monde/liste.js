// La liste des collections du monde, pour les pages du site : donnees/liste-c.json (écrit par
// outils/monde_web.py, en_colonnes) porte l'index et toutes les œuvres par colonnes, un quart
// plus léger que index.json et liste.json ensemble. Il est déplié ici en ce que les pages
// connaissent : l'index (musées, catégories, siècles, ids) et une ligne par œuvre
// [titre, artiste, date, année, catégorie, musée, ratio, vignette du musée s'il y en a une].
import { lien } from "./commun.js";
import { jsonGarde } from "./memoire.js";

export function deplier(c) {
  const { gabarits, artistes, noms, t, a, d, y, k, r, v, i, ...index } = c;
  const categories = c.c, cles = index.musees.map((m) => m.cle);
  const n = t.length, liste = new Array(n), ids = new Array(n);
  for (let j = 0; j < n; j++) {
    const musee = k[j], vu = v[j], id = i[j];
    const ligne = [t[j], artistes[a[j]], d[j], y[j], noms[categories[j]], musee, r[j]];
    if (vu) ligne.push(vu.startsWith("http") ? vu : gabarits[musee].replaceAll("{}", vu));
    liste[j] = ligne;
    ids[j] = id.startsWith("=") ? id.slice(1) : `${cles[musee]}-${id}`;
  }
  delete index.c;
  index.ids = ids;
  return { index, liste };
}

let charge;
/** { index, liste } : toutes les œuvres des collections du monde (une seule lecture par page). */
export function chargerCollections() {
  charge ??= jsonGarde(lien("monde/donnees/liste-c.json")).then(deplier);
  charge.catch(() => { charge = null; });
  return charge;
}
