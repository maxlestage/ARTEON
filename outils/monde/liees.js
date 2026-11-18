// Sur la page « Œuvres » d'une exposition : toutes ses œuvres. Après celles de la visite 3D
// (écrites dans la page), toutes les œuvres liées des collections du monde, de la plus
// ancienne à la plus récente : elles s'affichent d'elles-mêmes à mesure que l'on descend,
// sans bouton. Les données (monde/donnees/expo-<nom>.json) sont écrites par
// outils/monde_web.py : [identifiant, titre, artiste, date, vignette du musée ou rien].
import { echapper, lien, lienPage, nombre, t } from "./commun.js";
import { jsonGarde } from "./memoire.js";

const PAR_LOT = 240;
const zone = document.getElementById("oeuvres-liees");

async function ouvrir() {
  const nom = zone.dataset.exposition;
  const grille = zone.querySelector(".monde-grille"), compte = zone.querySelector(".monde-compte");
  let oeuvres;
  try {
    oeuvres = await jsonGarde(lien(`monde/donnees/expo-${nom}.json`));
  } catch {
    compte.textContent = t("Les collections n'ont pas répondu. Réessayer plus tard.", "The collections did not answer. Try again later.", "Las colecciones no respondieron. Inténtelo más tarde.");
    return;
  }
  const inconnu = t("Artiste inconnu", "Unknown artist", "Artista desconocido");
  const carte = ([id, titre, artiste, date, vu]) => `<a class="monde-carte" href="${lienPage(`monde/#/oeuvre/${id}`)}"><img loading="lazy" decoding="async" src="${echapper(vu || lien(`monde/v/${id}.webp`))}" alt=""><b>${echapper(titre)}</b><span>${echapper([artiste === "Artiste inconnu" ? inconnu : artiste, date].filter(Boolean).join(" · "))}</span></a>`;
  let montrees = 0;
  const fin = document.createElement("div");
  zone.append(fin);
  const plus = () => {
    if (montrees >= oeuvres.length) return;
    grille.insertAdjacentHTML("beforeend", oeuvres.slice(montrees, montrees + PAR_LOT).map(carte).join(""));
    montrees = Math.min(montrees + PAR_LOT, oeuvres.length);
    compte.textContent = montrees < oeuvres.length
      ? `${nombre(montrees)} / ${nombre(oeuvres.length)} ${t("œuvres affichées : la suite arrive en descendant", "works shown: more appear as you scroll", "obras mostradas: aparecen más al bajar")}`
      : `${nombre(oeuvres.length)} ${t("œuvres, toutes affichées", "works, all shown", "obras, todas mostradas")}`;
  };
  plus();
  // La suite, dès que la fin de la grille approche de l'écran.
  new IntersectionObserver((e) => { if (e.some((x) => x.isIntersecting)) plus(); }, { rootMargin: "1500px" }).observe(fin);
}

if (zone) ouvrir();
