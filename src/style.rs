//! Le style d'ARTEON, écrit dans chaque page avec `Page::style` (la Content-Security-
//! Policy l'autorise par son empreinte). Les couleurs sont celles de Van Gogh : bleu
//! outremer, jaune de chrome, orange et vert ; les animations s'arrêtent pour les
//! visiteurs qui préfèrent moins de mouvement.

pub const CSS: &str = r#"
:root {
  --red: #d4572a; --yellow: #f2b705; --blue: #1d3f8f; --green: #2f6b4f; --ink: #0f1a3a; --paper: #f4ecd8;
  --accent: #1d3f8f; --accent-fg: #fff; --radius: 14px;
  --font: "Inter", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
/* Le thème de Paquets centre le body dans 72rem : ici, l'en-tête et le pied de page
   prennent toute la largeur, et c'est `.page` qui centre le contenu. */
body { margin: 0; max-width: none; padding: 0; overflow-x: hidden; }
.page { max-width: 1180px; margin: 0 auto; padding: 0 1.25rem 4rem; }
.muted { color: var(--muted); }
.kicker { text-transform: uppercase; letter-spacing: .18em; font-size: .8rem; color: var(--accent); font-weight: 700; margin: 0; }
.lead { font-size: 1.15rem; line-height: 1.6; max-width: 38rem; }
.tag { display: inline-block; font-size: .78rem; padding: .2rem .6rem; border: 1px solid var(--line); border-radius: 999px; color: var(--muted); }
h2.section-title { font-size: clamp(1.6rem, 3vw, 2.3rem); margin: 4rem 0 1.5rem; }

/* ─── En-tête ─────────────────────────────────────────────────────────── */
.site-header { position: sticky; top: 0; z-index: 10; display: flex; justify-content: space-between; align-items: center;
  padding: .8rem 1.5rem; backdrop-filter: blur(12px); background: color-mix(in srgb, var(--bg) 82%, transparent); }
.site-header nav { display: flex; gap: 1.2rem; }
.site-header nav a { color: var(--fg); text-decoration: none; position: relative; padding: .2rem 0; }
.site-header nav a::after { content: ""; position: absolute; left: 0; bottom: -2px; height: 2px; width: 100%;
  background: var(--accent); transform: scaleX(0); transform-origin: left; transition: transform .3s ease; }
.site-header nav a:hover::after, .site-header nav a.current::after { transform: scaleX(1); }
.site-header nav { align-items: center; }
.nav-menu { position: relative; }
.nav-menu summary { list-style: none; cursor: pointer; padding: .2rem 0; position: relative; }
.nav-menu summary::-webkit-details-marker { display: none; }
.nav-menu summary::after { content: " ▾"; font-size: .8em; }
.nav-menu summary.current { box-shadow: inset 0 -2px 0 var(--accent); }
.nav-menu ul { position: absolute; right: 0; top: calc(100% + .6rem); z-index: 20; min-width: 15rem; list-style: none; margin: 0; padding: .5rem;
  background: var(--card); border: 1px solid var(--line); border-radius: 12px; box-shadow: 0 20px 40px -20px rgba(0,0,0,.45); }
.nav-menu li { display: flex; justify-content: space-between; align-items: baseline; gap: 1rem; padding: .45rem .6rem; border-radius: 8px; }
.nav-menu li:hover { background: color-mix(in srgb, var(--accent) 10%, transparent); }
.nav-menu li a::after { display: none; }
.nav-menu li a.current { font-weight: 700; color: var(--accent); }
.nav-artist { font-size: .78rem; color: var(--muted); white-space: nowrap; }
.nav-menu ul { max-height: min(70vh, 34rem); overflow-y: auto; overscroll-behavior: contain; }
.styles { display: flex; flex-wrap: wrap; gap: .45rem; margin: 0 0 2rem; }
.styles .chip { padding: .3rem .8rem; border-radius: 999px; font-size: .85rem; text-decoration: none; color: var(--fg);
  border: 1px solid color-mix(in srgb, var(--accent) 55%, transparent); background: color-mix(in srgb, var(--accent) 10%, transparent); }
.styles .chip:hover { background: color-mix(in srgb, var(--accent) 22%, transparent); }
.ex-card { scroll-margin-top: 5rem; }
.ex-artists { margin: 0; font-weight: 600; }
.ex-intro { display: -webkit-box; -webkit-line-clamp: 4; -webkit-box-orient: vertical; overflow: hidden; }
.logo { display: inline-flex; align-items: center; gap: .6rem; color: var(--fg); text-decoration: none; letter-spacing: .2em; }
.logo-mark { position: relative; width: 28px; height: 28px; border-radius: 7px; display: inline-block; overflow: hidden;
  background: radial-gradient(circle at 70% 30%, #3d63c9, var(--blue) 55%, var(--ink)); }
.logo-mark span { position: absolute; border-radius: 50%; }
.logo-mark .m-sun { left: 18%; top: 18%; width: 46%; height: 46%; background: var(--yellow);
  box-shadow: 0 0 0 3px rgba(242,183,5,.35), 0 0 0 6px rgba(242,183,5,.15); }
.logo-mark .m-star { right: 14%; bottom: 16%; width: 18%; height: 18%; background: #fff6c9; }
.logo:hover .logo-mark { animation: wobble .6s ease; }

/* ─── Hero ────────────────────────────────────────────────────────────── */
.hero { position: relative; display: grid; grid-template-columns: 1.1fr 1fr; gap: 2rem; align-items: center;
  text-align: left; padding: 4rem 0 2rem; min-height: 72vh; }
.hero-title { font-size: clamp(2.6rem, 6vw, 5rem); letter-spacing: .02em; margin: .3rem 0; line-height: 1.02; }
.hero-title .word { display: inline-block; white-space: nowrap; margin-right: .28em; }
.hero-title .letter { display: inline-block; opacity: 0; animation: drop .8s cubic-bezier(.2,.9,.3,1.3) forwards; }
.hero-title .word:nth-child(3n+1) .letter { color: var(--accent); }
.hero-title .word:nth-child(3n+2) .letter { color: var(--fg); }
.hero-title .word:nth-child(3n) .letter { color: var(--yellow); text-shadow: 0 2px 0 rgba(15,26,58,.25); }
.hero-sub { font-family: Georgia, "Times New Roman", serif; font-style: italic; font-size: clamp(1.2rem, 2.2vw, 1.6rem); margin: .2rem 0 .8rem; color: var(--red); }
.hero .row { justify-content: flex-start; gap: .8rem; }
.hero-visual { position: relative; min-height: 380px; perspective: 900px; }
.scene { display: block; width: 100%; border-radius: var(--radius); }
.scene-showcase { height: 420px; background: radial-gradient(circle at 50% 35%, #2a3f7a, #0f1a3a 72%);
  box-shadow: 0 30px 80px -30px rgba(29,63,143,.7); }
.floating { position: absolute; z-index: -1; opacity: .85; animation: float 9s ease-in-out infinite; border-radius: 50%; }
.f1 { width: 130px; height: 130px; background: radial-gradient(circle, #ffe27a, var(--yellow) 60%, rgba(242,183,5,0) 72%); top: 4%; left: -50px; }
.f2 { width: 60px; height: 60px; background: var(--blue); opacity: .7; bottom: 10%; left: 42%; animation-delay: -3s; }
.f3 { width: 180px; height: 14px; border-radius: 999px; background: var(--green); opacity: .6; top: 16%; right: 46%; animation-delay: -6s; }

/* Le cube CSS 3D, posé sur la vitrine. */
.cube-stage { position: absolute; right: -10px; top: -30px; width: 90px; height: 90px; perspective: 600px; }
.cube { position: relative; width: 100%; height: 100%; transform-style: preserve-3d; animation: cube 12s linear infinite; }
.cube .face { position: absolute; inset: 0; border: 3px solid var(--ink); opacity: .95; }
.cube .front  { background: var(--red);    transform: translateZ(45px); }
.cube .back   { background: var(--blue);   transform: rotateY(180deg) translateZ(45px); }
.cube .right  { background: var(--yellow); transform: rotateY(90deg) translateZ(45px); }
.cube .left   { background: var(--paper);  transform: rotateY(-90deg) translateZ(45px); }
.cube .top    { background: var(--paper);  transform: rotateX(90deg) translateZ(45px); }
.cube .bottom { background: var(--ink);    transform: rotateX(-90deg) translateZ(45px); }

/* ─── Sections ────────────────────────────────────────────────────────── */
.stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr)); gap: 1rem; margin-top: 2rem; }
.stat { padding: 1.4rem; border-left: 6px solid var(--accent); background: var(--card); border-radius: 0 var(--radius) var(--radius) 0; }
.stat:nth-child(2) { border-color: var(--yellow); } .stat:nth-child(3) { border-color: var(--red); } .stat:nth-child(4) { border-color: var(--green); }
.stat strong { display: block; font-size: 2.4rem; }
.feature { transition: transform .35s ease, box-shadow .35s ease; }
.feature:hover { transform: translateY(-6px) rotateX(4deg); box-shadow: 0 20px 40px -24px rgba(0,0,0,.45); }
.dot { display: inline-block; width: 18px; height: 18px; }
.dot { border-radius: 50%; }
.dot.red { background: var(--red); } .dot.yellow { background: var(--yellow); } .dot.blue { background: var(--blue); } .dot.green { background: var(--green); }
.timeline { display: grid; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); gap: 1.2rem; position: relative; }
.step { position: relative; padding: 1.4rem; border: 1px solid var(--line); border-radius: var(--radius); background: var(--card); }
.step-number { font-size: 3rem; font-weight: 800; color: transparent; -webkit-text-stroke: 1.5px var(--accent); }
.step-works { display: flex; gap: .4rem; margin: .4rem 0 .8rem; flex-wrap: wrap; }
.step-works img { height: 52px; width: auto; border-radius: 3px; box-shadow: 0 6px 14px -8px rgba(0,0,0,.6); transition: transform .3s ease; }
.step-works a:hover img { transform: translateY(-3px) scale(1.06); }
.preview-strip { display: flex; gap: 1.4rem; overflow-x: auto; padding: 1rem .2rem 2rem; perspective: 1000px; align-items: flex-start; }
.tilt { flex: 0 0 auto; display: flex; flex-direction: column; gap: .5rem; color: var(--fg); text-decoration: none;
  transition: transform .45s cubic-bezier(.2,.8,.2,1); }
.tilt img { height: 220px; width: auto; display: block; border: 8px solid #a8803a; box-shadow: 0 18px 30px -18px rgba(0,0,0,.7); }
.tilt-caption { font-size: .85rem; color: var(--muted); max-width: 16rem; }
.tilt:hover { transform: rotateY(-14deg) rotateX(6deg) translateZ(30px) scale(1.04); }

/* ─── Catalogue ───────────────────────────────────────────────────────── */
.frame { padding: 10px; background: linear-gradient(135deg, #c99b45, #8a6326 45%, #d6ad5a 70%, #8a6326); border-radius: 3px;
  box-shadow: 0 18px 34px -20px rgba(0,0,0,.7); }
.frame img { display: block; width: 100%; height: auto; }
.small { font-size: .85rem; }
.art-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr)); gap: 1.6rem; }
.art-card { display: flex; flex-direction: column; gap: .3rem; }
.art-card .frame { transition: transform .4s ease; }
.art-card:hover .frame { transform: translateY(-4px) rotate(-1deg); }
.page-intro { padding: 3rem 0 1.5rem; }
.viewer { position: relative; border-radius: var(--radius); overflow: hidden; border: 1px solid var(--line); }
.scene-gallery { height: min(72vh, 620px); background: linear-gradient(#1a1b22, #0b0b10); }

/* ─── Les commandes de la galerie et le plein écran ───────────────────── */
.viewer-shell { position: relative; }
.scene-gallery { cursor: zoom-in; }
.viewer-shell button { font: inherit; border: 0; cursor: pointer; color: #fff; -webkit-tap-highlight-color: transparent; }
.viewer-full { position: absolute; top: 14px; right: 14px; padding: .55rem .95rem; border-radius: 999px; font-size: .9rem !important;
  background: rgba(10,14,30,.62); backdrop-filter: blur(8px); box-shadow: 0 8px 24px -10px rgba(0,0,0,.6); }
.viewer-full:hover { background: rgba(10,14,30,.85); }
.viewer-caption { position: absolute; left: 14px; bottom: 14px; max-width: min(28rem, calc(100% - 13rem)); padding: .75rem 1rem;
  border-radius: 14px; color: #fff; background: rgba(10,14,30,.62); backdrop-filter: blur(10px);
  opacity: 0; transform: translateY(8px); transition: opacity .4s ease, transform .4s ease; pointer-events: none; }
.has-art .viewer-caption { opacity: 1; transform: none; pointer-events: auto; }
.cap-title { margin: 0; font-weight: 700; font-size: 1.05rem; line-height: 1.3; }
.cap-meta { margin: .15rem 0 .5rem; font-size: .82rem; color: #d4d6e2; line-height: 1.4; }
.viewer-caption .cap-hd { display: none; }
.has-hd .viewer-caption .cap-hd { display: inline-block; }
.cap-hd { padding: .35rem .8rem; border-radius: 999px; font-size: .82rem !important; background: var(--accent); }
.viewer-controls { position: absolute; right: 14px; bottom: 14px; display: flex; gap: .4rem; }
.viewer-controls button { width: 44px; height: 44px; border-radius: 50%; font-size: 1.25rem; line-height: 1;
  background: rgba(10,14,30,.62); backdrop-filter: blur(8px); }
.viewer-controls button:hover { background: rgba(10,14,30,.85); }
.viewer-controls .ctl-exit { display: none; }
.is-paused .ctl-play { background: var(--accent); }

/* L'interface egui : un canvas transparent posé sur toute la scène. */
.egui-layer { position: absolute; inset: 0; width: 100%; height: 100%; display: block; outline: none; touch-action: none; }
.is-loading .egui-layer { visibility: hidden; }

/* Le chargement des données de l'exposition. */
.viewer-loading { display: none; position: absolute; inset: 0; margin: 0; align-items: center; justify-content: center;
  color: #e8e9f0; font-size: .95rem; text-align: center; padding: 1rem; pointer-events: none; }
.is-loading .viewer-loading { display: flex; }
.is-loading .viewer-controls, .is-loading .viewer-caption { visibility: hidden; }
.viewer-loading::before { content: ""; width: 18px; height: 18px; margin-right: .7rem; border-radius: 50%;
  border: 3px solid rgba(255,255,255,.25); border-top-color: var(--accent); animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
/* Le bouton de la visite libre est mis en avant ; sa couleur dit le mode actif. */
.is-free .ctl-mode { background: var(--accent) !important; }
.is-free .ctl-play { display: none; }
.is-free .scene-gallery { cursor: grab; }
.is-free .scene-gallery:active { cursor: grabbing; }
/* Les joysticks tactiles : un repère discret, seulement en visite libre sur écran tactile. */
.viewer-sticks { display: none; }
@media (pointer: coarse) {
  .is-free .viewer-sticks { display: flex; position: absolute; inset: auto 0 18px 0; justify-content: space-between; padding: 0 22px; pointer-events: none; }
  .stick { width: 86px; height: 86px; border-radius: 50%; border: 2px solid rgba(255,255,255,.55); background: rgba(10,14,30,.25);
    color: rgba(255,255,255,.8); font-size: .7rem; display: flex; align-items: center; justify-content: center; }
  .is-free .viewer-caption { bottom: 120px; }
}

.viewer-shell.is-fullscreen { position: fixed; inset: 0; z-index: 1000; background: #000; }
.is-fullscreen .scene-gallery { height: 100vh; height: 100dvh; border-radius: 0; cursor: pointer; }
.is-fullscreen .viewer-full { display: none; }
.is-fullscreen .viewer-controls .ctl-exit { display: block; }
.is-fullscreen .viewer-controls { top: max(14px, env(safe-area-inset-top)); bottom: auto; right: max(14px, env(safe-area-inset-right)); }
.is-fullscreen .viewer-caption { left: max(14px, env(safe-area-inset-left)); bottom: max(18px, env(safe-area-inset-bottom)); max-width: min(34rem, calc(100% - 28px)); }
body.viewer-open { overflow: hidden; }

.viewer-hd { display: none; }
.hd-open .viewer-hd { position: fixed; inset: 0; z-index: 1001; display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: .8rem; padding: max(16px, env(safe-area-inset-top)) 16px max(16px, env(safe-area-inset-bottom)); background: rgba(0,0,0,.94); }
.hd-img { max-width: 100%; max-height: calc(100dvh - 9rem); object-fit: contain; box-shadow: 0 30px 80px -20px rgba(0,0,0,.9); }
.hd-title { margin: 0; color: #e8e8ee; text-align: center; font-size: .92rem; }
.hd-close { padding: .55rem 1.1rem; border-radius: 999px; background: rgba(255,255,255,.14) !important; }
@media (max-width: 640px) {
  .viewer-caption { max-width: calc(100% - 28px); bottom: 70px; }
  .viewer-full { top: 10px; right: 10px; }
}
.room-legend { margin-top: 2rem; }

/* ─── Accueil du musée : couvertures, expositions ─────────────────────── */
.hero-covers { min-height: 440px; perspective: 1200px; }
.cover-frame { position: absolute; padding: 9px; background: linear-gradient(135deg, #c99b45, #8a6326 45%, #d6ad5a 70%, #8a6326);
  box-shadow: 0 30px 60px -28px rgba(0,0,0,.7); transition: transform .5s cubic-bezier(.2,.8,.2,1); animation: rise 1s cubic-bezier(.2,.8,.2,1) both; }
.cover-frame img { display: block; width: 100%; height: auto; }
.cover-frame.c0 { width: 58%; left: 4%; top: 8%; transform: rotateY(14deg) rotate(-4deg); z-index: 2; }
.cover-frame.c1 { width: 50%; right: 2%; top: 26%; transform: rotateY(-16deg) rotate(3deg); z-index: 3; animation-delay: .15s; }
.cover-frame.c2 { width: 36%; left: 30%; bottom: 0; transform: rotate(-1deg); z-index: 1; animation-delay: .3s; }
.hero-covers:hover .c0 { transform: rotateY(4deg) rotate(-2deg) translateY(-6px); }
.hero-covers:hover .c1 { transform: rotateY(-6deg) rotate(1deg) translateY(-6px); }
.ex-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr)); gap: 1.6rem; }
.ex-card { display: flex; flex-direction: column; border-radius: var(--radius); overflow: hidden; background: var(--card);
  border: 1px solid var(--line); border-top: 6px solid var(--accent); transition: transform .35s ease, box-shadow .35s ease; }
.ex-card:hover { transform: translateY(-6px); box-shadow: 0 28px 50px -30px rgba(0,0,0,.55); }
.ex-cover { display: block; height: 260px; overflow: hidden; background: #111; }
.ex-cover img { width: 100%; height: 100%; object-fit: cover; transition: transform .8s ease; }
.ex-card:hover .ex-cover img { transform: scale(1.05); }
.ex-body { padding: 1.3rem 1.4rem 1.5rem; display: flex; flex-direction: column; gap: .5rem; }
.ex-body h3 { margin: 0; font-size: 1.5rem; }
.ex-body h3 a { color: var(--fg); text-decoration: none; }
.ex-body .kicker { color: var(--accent); }
.ex-body .row { justify-content: flex-start; gap: .7rem; margin-top: .4rem; }

/* ─── Onglets d'une exposition et autres expositions ──────────────────── */
.ex-tabs { display: flex; gap: .4rem; flex-wrap: wrap; margin: 1.4rem 0 0; padding: .35rem; width: fit-content;
  border: 1px solid var(--line); border-radius: 999px; background: var(--card); }
.ex-tabs a { padding: .4rem 1rem; border-radius: 999px; color: var(--fg); text-decoration: none; font-size: .92rem; }
.ex-tabs a:hover { background: color-mix(in srgb, var(--accent) 12%, transparent); }
.ex-tabs a.current { background: var(--accent); color: #fff; }
.others-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); gap: 1.2rem; }
.other { position: relative; display: block; height: 200px; border-radius: var(--radius); overflow: hidden; color: #fff; text-decoration: none; }
.other img { width: 100%; height: 100%; object-fit: cover; transition: transform .8s ease; }
.other:hover img { transform: scale(1.06); }
.other-text { position: absolute; inset: auto 0 0 0; padding: 1rem 1.2rem; display: flex; flex-direction: column;
  background: linear-gradient(transparent, rgba(0,0,0,.75)); }
.other-text strong { font-size: 1.4rem; }
.slots-block h3 { font-size: 1rem; margin: 1.6rem 0 .6rem; }

/* ─── Ajouter une œuvre ───────────────────────────────────────────────── */
.how { margin-top: 1rem; }
.how-step .step-number { font-size: 2.2rem; }
.how-step code, .rules code { background: color-mix(in srgb, var(--accent) 10%, transparent); padding: .1rem .35rem; border-radius: 5px; }
.example { background: #0f1a3a; color: #f4ecd8; padding: 1.2rem 1.4rem; border-radius: var(--radius); overflow-x: auto; font-size: .92rem; line-height: 1.6;
  border-left: 6px solid var(--yellow); }
.rules { line-height: 1.7; margin-top: 1.5rem; }
.table-wrap { overflow-x: auto; border: 1px solid var(--line); border-radius: var(--radius); }
.slots { width: 100%; border-collapse: collapse; margin: 0; font-size: .92rem; }
.slots th, .slots td { padding: .6rem .8rem; border-bottom: 1px solid var(--line); text-align: left; }
.slots thead th { background: var(--card); }
.slots .taken { font-weight: 600; }
.slots .free { color: var(--green); }
.slots .free::before { content: "● "; }
.slots td .chip { display: inline-block; margin: .15rem .3rem .15rem 0; padding: .15rem .55rem; border-radius: 999px; border: 1px solid var(--line); font-size: .82rem; }
.slots td .chip.taken { font-weight: 600; background: color-mix(in srgb, var(--accent) 8%, transparent); }
/* Les sculptures du catalogue, posées sur un socle clair. */
.plinth { display: flex; align-items: flex-end; justify-content: center; min-height: 260px; padding: 1.2rem 1rem 0;
  border-radius: 6px; background: linear-gradient(#2b2d36, #15161c 72%, #8f8a80 72%, #6f6a62); }
.sculpture-img { max-height: 240px; width: auto; filter: drop-shadow(0 10px 14px rgba(0,0,0,.6)); }
.step-works .sculpture-img { height: 52px; filter: none; background: #2b2d36; border-radius: 3px; padding: 2px; }
.slots .door { color: var(--muted); font-style: italic; }

/* ─── Le grand pied de page ───────────────────────────────────────────── */
.site-footer { display: block; margin-top: 5rem; padding: 0; border: 0; background: #0b1430; color: #ece9e1; }
.site-footer a { color: #ece9e1; }
.site-footer .muted { color: #9a98a3; }
.footer-band { display: grid; grid-template-columns: 3fr 1fr 2fr 1fr 4fr 1fr 2fr 2fr; height: 14px; }
.footer-band span { transform-origin: bottom; animation: band 3.2s ease-in-out infinite; }
.marquee { overflow: hidden; border-bottom: 1px solid #262833; padding: 1.1rem 0; mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent); }
.marquee-track { display: flex; width: max-content; animation: marquee 40s linear infinite; }
.marquee:hover .marquee-track, .marquee:active .marquee-track { animation-play-state: paused; }
.marquee-item { padding: 0 2rem; white-space: nowrap; font-size: 1.05rem; color: inherit; text-decoration: none; }
.marquee-item strong { color: var(--yellow); }
.footer-grid { max-width: 1180px; margin: 0 auto; padding: 3.5rem 1.25rem 2rem; display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr; gap: 2.5rem; }
.footer-brand p { color: #b9b6c2; max-width: 22rem; line-height: 1.6; }
.footer-brand .logo { color: #fff; }
.footer-cta { margin-top: .6rem; animation: pulse 2.6s ease-in-out infinite; }
.footer-col h3 { font-size: .8rem; text-transform: uppercase; letter-spacing: .18em; color: #9a98a3; margin-bottom: 1rem; }
.footer-col ul { list-style: none; padding: 0; margin: 0; display: grid; gap: .55rem; }
.footer-wide { grid-column: span 2; }
.footer-wide ul { grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr)); gap: .45rem 1rem; }
.footer-3d { font-size: .8rem; color: var(--yellow); }
.footer-col a { text-decoration: none; background: linear-gradient(var(--yellow), var(--yellow)) 0 100% / 0 2px no-repeat; transition: background-size .3s ease; }
.footer-col a:hover { background-size: 100% 2px; }
.footer-legal { max-width: 1180px; margin: 0 auto; padding: 1.5rem 1.25rem 2.5rem; border-top: 1px solid #262833; font-size: .88rem; }

/* ─── Apparitions ─────────────────────────────────────────────────────── */
.reveal { opacity: 0; animation: rise .9s cubic-bezier(.2,.8,.2,1) forwards; }
@supports (animation-timeline: view()) {
  .reveal { animation: rise linear both; animation-timeline: view(); animation-range: entry 0% cover 30%; }
}

@keyframes drop { from { opacity: 0; transform: translateY(-60px) rotate(-12deg); } to { opacity: 1; transform: none; } }
@keyframes rise { from { opacity: 0; transform: translateY(28px); } to { opacity: 1; transform: none; } }
@keyframes float { 0%, 100% { transform: translate(0, 0) rotate(0); } 50% { transform: translate(18px, -22px) rotate(8deg); } }
@keyframes cube { from { transform: rotateX(-20deg) rotateY(0); } to { transform: rotateX(-20deg) rotateY(360deg); } }
@keyframes wobble { 0%, 100% { transform: rotate(0); } 30% { transform: rotate(-12deg); } 60% { transform: rotate(8deg); } }
@keyframes band { 0%, 100% { transform: scaleY(1); } 50% { transform: scaleY(2.2); } }
@keyframes marquee { from { transform: translateX(0); } to { transform: translateX(-50%); } }
@keyframes pulse { 0%, 100% { box-shadow: 0 0 0 0 rgba(242,183,5,.5); } 50% { box-shadow: 0 0 0 12px rgba(242,183,5,0); } }

@media (max-width: 820px) {
  .hero { grid-template-columns: 1fr; text-align: center; min-height: auto; }
  .hero .row { justify-content: center; }
  .hero { padding-top: 2rem; }
  /* Sur un petit écran, les formes décoratives passeraient sur le titre et les boutons. */
  .floating { display: none; }
  .cube-stage { width: 60px; height: 60px; right: 6px; top: -24px; }
  .cube .face { border-width: 2px; }
  .tilt img { height: 170px; }
  .hero-covers { min-height: 300px; margin-top: 1rem; }
  .cube .front { transform: translateZ(30px); } .cube .back { transform: rotateY(180deg) translateZ(30px); }
  .cube .right { transform: rotateY(90deg) translateZ(30px); } .cube .left { transform: rotateY(-90deg) translateZ(30px); }
  .cube .top { transform: rotateX(90deg) translateZ(30px); } .cube .bottom { transform: rotateX(-90deg) translateZ(30px); }
  .lead { margin-inline: auto; }
  .footer-grid { grid-template-columns: 1fr 1fr; }
  .site-header nav { gap: .7rem; font-size: .9rem; }
}
@media (max-width: 520px) { .footer-grid { grid-template-columns: 1fr; } .footer-wide { grid-column: auto; } .nav-long { display: none; } }
/* Sur un téléphone, le menu des expositions s'ouvre sous l'en-tête, sur toute la largeur. */
@media (max-width: 640px) {
  .nav-menu { position: static; }
  .nav-menu ul { left: 12px; right: 12px; top: calc(100% - .3rem); min-width: 0; }
}

/* ─── Le bâtiment ─────────────────────────────────────────────────────── */
.building-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr); gap: 2.5rem; align-items: start; }
.building-text p { line-height: 1.7; }
.building-text h3 { font-size: 1.6rem; margin: .2rem 0 .3rem; }
.building-text h4 { margin: 1.4rem 0 .3rem; color: var(--accent); }
.building-plan { margin: 0; padding: 1rem; border-radius: var(--radius); background: var(--card); border: 1px solid var(--line); }
.plan { width: 100%; height: auto; max-height: 70vh; display: block; }
.plan-room { stroke: #3a3530; stroke-width: .25; }
.plan-room.corridor { opacity: .75; }
.plan-door { stroke: var(--card); stroke-width: .45; }
.plan-art { stroke: var(--accent); stroke-width: .35; stroke-linecap: round; }
.plan-sculpture { fill: #fff; stroke: #3a3530; stroke-width: .15; }
.plan-number { fill: #3a3530; font-weight: 700; text-anchor: middle; font-family: var(--font); opacity: .7; }
.plan-entrance { fill: var(--accent); font-weight: 700; text-anchor: middle; font-family: var(--font); }
.plan-legend { margin: 1rem 0 0; padding-left: 1.4rem; display: grid; gap: .35rem; }
@media (max-width: 820px) { .building-grid { grid-template-columns: 1fr; gap: 1.5rem; } }

/* ─── La page d'une œuvre ─────────────────────────────────────────────── */
.art-card h3 a { color: inherit; text-decoration: none; }
.art-card h3 a:hover { color: var(--accent); }
.art-card > a.frame, .art-card > a.plinth { display: block; }
.art-card .more { margin-top: auto; padding-top: .3rem; font-weight: 600; color: var(--accent); text-decoration: none; }
.art-card .more:hover { text-decoration: underline; }
.work-byline { font-size: 1.15rem; margin: .3rem 0 0; color: var(--muted); }
.work-head { display: grid; grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr); gap: 2.5rem; align-items: start; margin-top: 1.5rem; }
.work-frame { display: block; }
.work-frame img { width: 100%; height: auto; display: block; }
.work-frame.plinth img { max-height: 520px; width: auto; }
.work-cartel .lead { margin-top: 0; }
.facts { display: grid; gap: .55rem; margin: 1.2rem 0 1.5rem; }
.fact { display: grid; grid-template-columns: 8.5rem 1fr; gap: .8rem; padding-bottom: .55rem; border-bottom: 1px solid var(--line); }
.fact dt { color: var(--muted); font-size: .85rem; text-transform: uppercase; letter-spacing: .08em; }
.fact dd { margin: 0; }
.themes { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: 1.2rem; }
.themes .chip { padding: .2rem .7rem; border-radius: 999px; font-size: .82rem; background: color-mix(in srgb, var(--accent) 12%, transparent); color: var(--fg); }
.notice { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem 3rem; margin-top: 3rem; }
.notice-part h2 { font-size: 1.35rem; margin: 0 0 .6rem; padding-top: .8rem; border-top: 3px solid var(--accent); display: inline-block; }
.notice-part p, .notice-part li, .artist-card p { line-height: 1.7; }
.details { padding-left: 1.2rem; margin: 0; }
.details li { margin-bottom: .45rem; }
.anecdote { grid-column: 1 / -1; padding: 1.2rem 1.4rem; border-radius: var(--radius); line-height: 1.6;
  background: color-mix(in srgb, var(--accent) 10%, var(--card)); border-left: 4px solid var(--accent); }
.artist-list { display: grid; gap: 1.2rem; }
.artist-list:has(> :nth-child(2)) { grid-template-columns: repeat(auto-fill, minmax(min(100%, 26rem), 1fr)); }
.artist-card h3 { margin-top: 0; }
.artist-card h4 { margin: 1rem 0 .3rem; color: var(--accent); }
.work-nav { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 3.5rem; }
.work-nav a { display: flex; flex-direction: column; gap: .2rem; padding: 1rem 1.2rem; border: 1px solid var(--line); border-radius: var(--radius);
  text-decoration: none; color: var(--fg); background: var(--card); }
.work-nav a:hover { border-color: var(--accent); }
.work-nav .next { text-align: right; }
@media (max-width: 820px) {
  .work-head, .notice { grid-template-columns: 1fr; gap: 1.5rem; }
  .fact { grid-template-columns: 1fr; gap: .1rem; }
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation: none !important; transition: none !important; }
  .reveal, .hero-title .letter { opacity: 1; }
}
"#;
