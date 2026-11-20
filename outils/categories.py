#!/usr/bin/env python3
"""Les catégories plus fines des collections du monde.

La moisson (outils/collections_monde.py) range chaque œuvre dans une grande catégorie d'après
ce que dit son musée : « Peinture », « Estampe », « Sculpture », « Céramique »… Seize en tout,
dont une (« Estampe ») réunit plus du tiers des œuvres. Ce module en tire des catégories plus
précises, d'après le titre, la technique et l'origine de l'œuvre : l'estampe japonaise,
l'aquarelle, le pastel, la miniature et l'enluminure, les rouleaux et paravents, la porcelaine,
les vases grecs, la sculpture par matière, la dentelle, le costume, les masques, l'ivoire, le
jade, la laque… Les fiches gardées en cache n'ont pas besoin d'être redemandées aux musées :
le site (outils/monde_web.py) applique `affiner` au moment d'écrire ses pages.

Les noms affichés en anglais et en espagnol sont dans outils/monde/app.js (CATEGORIES) et
ios/ARTEON.swiftpm/Language.swift : une catégorie de plus ici s'ajoute là aussi.

  python3 outils/categories.py        vérifie les règles sur quelques œuvres
"""

import re
# Chaque règle : (nom, catégories d'origine, motif sur le titre, motif sur la technique, motif
# sur l'origine). Une règle s'applique si l'œuvre est dans une des catégories d'origine, si son
# origine convient (quand la règle en demande une), et si son titre ou sa technique convient.
# La première qui s'applique gagne. Les motifs sont en anglais, la langue des fiches des musées.
OBJETS = {"Sculpture", "Céramique", "Textile et costume", "Mobilier", "Verre", "Bijoux", "Armes et armures",
          "Monnaies et médailles", "Instruments de musique", "Orfèvrerie et métal", "Autres objets"}
PAPIER = {"Dessin", "Peinture"}
REGLES = [
    ("Estampe japonaise", {"Estampe"}, None, r"woodblock|nishiki|surimono|ukiyo", r"japan"),
    ("Affiche", {"Estampe", "Dessin", "Autres objets"}, r"\bposters?\b|\baffiches?\b", r"\bposter\b", None),
    ("Dessin d'architecture", {"Dessin"}, r"\belevations?\b|floor plan|\bplan of\b|, (plan|section|perspective)\b|\bperspective (view|rendering)\b", None, None),
    ("Miniature et enluminure", PAPIER | {"Livres et manuscrits", "Autres objets"},
     r"\bminiature\b|illuminat|\bleaf from\b|\bfolio from\b|\bpage from\b|book of hours|\bkoran\b|qur'an|shahnama|ragamala|\bmissal\b|\bpsalter\b|\bantiphonary\b|\bgradual\b",
     r"illuminat|opaque watercolor|on vellum|on parchment|watercolor on ivory", None),
    ("Rouleaux et paravents", PAPIER | {"Autres objets"}, r"handscroll|hanging scroll|folding screen", r"hanging scroll|handscroll|folding screen|\bscreens?\b", None),
    ("Icônes et retables", {"Peinture"}, r"\bicon\b|altarpiece|\bretable\b|triptych|polyptych|diptych|predella", None, None),
    # L'aquarelle et le pastel : la technique principale (nommée d'abord), pas une touche parmi
    # d'autres (« graphite, with touches of watercolor »).
    ("Aquarelle", PAPIER, None, r"^.{0,24}(watercolou?r|gouache|aquarelle)", None),
    ("Pastel", PAPIER, None, r"^.{0,16}\bpastel", None),
    ("Éventails", OBJETS | {"Peinture"}, r"\bfans?\b(?! ?-?shaped)", r"\bfan painting\b|^fan\b", None),
    ("Masques", OBJETS, r"\bmasks?\b", None, None),
    ("Vannerie", {"Autres objets", "Textile et costume", "Mobilier"}, r"\bbaskets?\b", r"basketry|\bcoiled\b|plaited", None),
    ("Mosaïques et vitraux", OBJETS | {"Peinture"}, r"\bmosaic|stained[- ]glass", r"\bmosaic|stained glass|leaded glass", None),
    ("Horlogerie", OBJETS, r"\bclocks?\b|\bwatch\b|\bwatches\b|sundial|hourglass", None, None),
    ("Art funéraire égyptien", OBJETS | {"Livres et manuscrits"}, r"shabti|shawabty|ushabti|canopic|\bmummy\b|\bcoffin\b|sarcophag|book of the dead", None, None),
    ("Sceaux et amulettes", OBJETS, r"\bseals?\b|amulet|scarab|talisman", None, None),
    ("Tapisseries et tapis", {"Textile et costume"}, r"tapestr|carpet|\brugs?\b", r"knotted pile", None),
    ("Costume et mode", {"Textile et costume"},
     r"\bdress\b|costume|kimono|\brobes?\b|\bcoat\b|\bhats?\b|\bshoes?\b|\bgown\b|\bjacket\b|\bcape\b|\bskirt\b|\bbodice\b|waistcoat|\bsuit\b|ensemble|\bshawl\b|\bcap\b|\bbonnet\b|\bgloves?\b|\bslippers?\b|\bboots?\b|\bvest\b|\btunic\b|\bshirt\b|\bsash\b|\bobi\b|\bsari\b|poncho|\bmantle\b|\bcloak\b|\bparasol\b|\bevening\b|\bwedding\b",
     None, None),
    ("Dentelle et broderie", {"Textile et costume"}, r"\blace\b|embroider|sampler|needlework|\bquilt\b", r"\blace\b|embroider|needle", None),
    ("Vases grecs", {"Céramique"}, r"amphora|kylix|krater|lekythos|hydria|oinochoe|skyphos|alabastron|aryballos|\bpyxis\b|kantharos|pelike|stamnos|black-figure|red-figure", r"black-figure|red-figure", None),
    ("Porcelaine", {"Céramique"}, r"porcelain", r"porcelain", None),
    ("Laque", OBJETS, r"lacquer", r"lacquer", None),
    ("Ivoire", OBJETS, r"\bivory\b|netsuke", r"\bivory\b(?! (laid|wove|paper|tinted|colored|china|tone))", None),
    ("Jade et pierres dures", OBJETS, r"\bjade\b|rock crystal|\bagate\b|\bcameo\b|intaglio", r"\bjade\b|nephrite|jadeite|rock crystal|\bagate\b|carnelian|lapis|chalcedony|sardonyx", None),
    ("Sculpture en bronze", {"Sculpture"}, None, r"\bbronze\b", None),
    ("Sculpture en marbre et en pierre", {"Sculpture"}, None, r"marble|limestone|sandstone|\bstone\b|alabaster|granite|schist|basalt", None),
    ("Sculpture en bois", {"Sculpture"}, None, r"\bwood\b|walnut|\boak\b|limewood|boxwood", None),
    ("Sculpture en terre cuite", {"Sculpture"}, None, r"terracotta|earthenware|stoneware|\bclay\b|porcelain|\bceramic\b", None),
]
REGLES = [(n, o, t and re.compile(t), m and re.compile(m), g and re.compile(g)) for n, o, t, m, g in REGLES]

def affiner(categorie, titre, technique, origine):
    """La catégorie plus fine d'une œuvre, ou sa catégorie d'origine."""
    t, m, g = (titre or "").lower(), (technique or "").lower(), (origine or "").lower()
    for nom, origines, mt, mm, mg in REGLES:
        if categorie not in origines or (mg and not mg.search(g)):
            continue
        if (mt and mt.search(t)) or (mm and mm.search(m)):
            return nom
    return categorie


if __name__ == "__main__":
    essais = [
        ("Estampe", "The Great Wave", "Color woodblock print; oban", "Japan", "Estampe japonaise"),
        ("Estampe", "Melencolia I", "Engraving", "Germany", "Estampe"),
        ("Dessin", "Bridge at Caracas", "transparent watercolor and graphite", "", "Aquarelle"),
        ("Dessin", "Seated Man", "Graphite, with touches of watercolor, on ivory wove paper", "", "Dessin"),
        ("Dessin", "Two Monks", "Red chalk on ivory laid paper", "", "Dessin"),
        ("Dessin", "Dancer", "pastel on wove paper", "", "Pastel"),
        ("Peinture", "Leaf from a Book of Hours", "ink, tempera and gold on vellum", "", "Miniature et enluminure"),
        ("Peinture", "Maharaja in Durbar", "Opaque watercolor and gold on paper", "India", "Miniature et enluminure"),
        ("Peinture", "Bamboo in Wind", "hanging scroll; ink on silk", "China", "Rouleaux et paravents"),
        ("Peinture", "Water Lilies", "Oil on canvas", "France", "Peinture"),
        ("Sculpture", "The Thinker", "Bronze", "", "Sculpture en bronze"),
        ("Sculpture", "Head of a Woman", "Marble", "", "Sculpture en marbre et en pierre"),
        ("Céramique", "Cup and Saucer", "Soft-paste porcelain, polychrome enamels", "", "Porcelaine"),
        ("Céramique", "Red-Figure Kylix (Drinking Cup)", "ceramic", "", "Vases grecs"),
        ("Textile et costume", "Lace Edging", "Bobbin lace", "", "Dentelle et broderie"),
        ("Textile et costume", "Woman's Dress", "silk", "", "Costume et mode"),
        ("Autres objets", "Standing Woman", "ivory", "", "Ivoire"),
        ("Autres objets", "Shawabty of Tashery", "painted tamarisk", "Egypt", "Art funéraire égyptien"),
        ("Photographie", "Mask", "Gelatin silver print", "", "Photographie"),
    ]
    for categorie, titre, technique, origine, attendue in essais:
        trouvee = affiner(categorie, titre, technique, origine)
        assert trouvee == attendue, f"{titre} : {trouvee}, et non {attendue}"
    print(f"{len(essais)} œuvres bien rangées, {len(REGLES)} catégories plus fines")
