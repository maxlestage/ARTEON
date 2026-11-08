//! Le style d'ARTEON, écrit dans chaque page avec `Page::style` (la Content-Security-
//! Policy l'autorise par son empreinte). Les couleurs sont celles de De Stijl ; les
//! animations s'arrêtent pour les visiteurs qui préfèrent moins de mouvement.

pub const CSS: &str = r#"
:root {
  --red: #c82026; --yellow: #f2c828; --blue: #1c3a8c; --ink: #161616; --paper: #f2efe6;
  --accent: #c82026; --accent-fg: #fff; --radius: 14px;
  --font: "Inter", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; overflow-x: hidden; }
.page { max-width: 1180px; margin: 0 auto; padding: 0 1.25rem 4rem; }
.muted { color: var(--muted); }
.kicker { text-transform: uppercase; letter-spacing: .18em; font-size: .8rem; color: var(--accent); font-weight: 700; margin: 0; }
.lead { font-size: 1.15rem; line-height: 1.6; max-width: 38rem; }
.tag { display: inline-block; font-size: .78rem; padding: .2rem .6rem; border: 1px solid var(--line); border-radius: 999px; color: var(--muted); }
.section-title { font-size: clamp(1.6rem, 3vw, 2.3rem); margin: 4rem 0 1.5rem; }

/* ─── En-tête ─────────────────────────────────────────────────────────── */
.site-header { position: sticky; top: 0; z-index: 10; display: flex; justify-content: space-between; align-items: center;
  padding: .8rem 1.5rem; backdrop-filter: blur(12px); background: color-mix(in srgb, var(--bg) 82%, transparent); }
.site-header nav { display: flex; gap: 1.2rem; }
.site-header nav a { color: var(--fg); text-decoration: none; position: relative; padding: .2rem 0; }
.site-header nav a::after { content: ""; position: absolute; left: 0; bottom: -2px; height: 2px; width: 100%;
  background: var(--accent); transform: scaleX(0); transform-origin: left; transition: transform .3s ease; }
.site-header nav a:hover::after, .site-header nav a.current::after { transform: scaleX(1); }
.logo { display: inline-flex; align-items: center; gap: .6rem; color: var(--fg); text-decoration: none; letter-spacing: .2em; }
.logo-mark { position: relative; width: 26px; height: 26px; background: var(--paper); border: 2px solid var(--ink); display: inline-block; }
.logo-mark span { position: absolute; }
.logo-mark .m-r { left: 30%; top: 0; right: 0; bottom: 35%; background: var(--red); border-left: 2px solid var(--ink); border-bottom: 2px solid var(--ink); }
.logo-mark .m-b { left: 0; bottom: 0; width: 30%; height: 35%; background: var(--blue); }
.logo-mark .m-y { right: 0; bottom: 0; width: 22%; height: 25%; background: var(--yellow); border-left: 2px solid var(--ink); }
.logo:hover .logo-mark { animation: wobble .6s ease; }

/* ─── Hero ────────────────────────────────────────────────────────────── */
.hero { position: relative; display: grid; grid-template-columns: 1.1fr 1fr; gap: 2rem; align-items: center;
  text-align: left; padding: 4rem 0 2rem; min-height: 72vh; }
.hero-title { font-size: clamp(3.5rem, 11vw, 8rem); letter-spacing: .08em; margin: .3rem 0; line-height: 1; }
.hero-title span { display: inline-block; opacity: 0; animation: drop .8s cubic-bezier(.2,.9,.3,1.3) forwards; }
.hero-title span:nth-child(3n+1) { color: var(--red); }
.hero-title span:nth-child(3n+2) { color: var(--fg); }
.hero-title span:nth-child(3n) { color: var(--blue); }
.hero .row { justify-content: flex-start; gap: .8rem; }
.hero-visual { position: relative; min-height: 380px; perspective: 900px; }
.scene { display: block; width: 100%; border-radius: var(--radius); }
.scene-showcase { height: 380px; background: radial-gradient(circle at 50% 40%, #2b2f45, #0d0e14 70%);
  box-shadow: 0 30px 80px -30px rgba(28,58,140,.6); }
.floating { position: absolute; z-index: -1; opacity: .85; animation: float 9s ease-in-out infinite; }
.f1 { width: 120px; height: 120px; background: var(--red); top: 8%; left: -40px; }
.f2 { width: 70px; height: 70px; background: var(--yellow); bottom: 10%; left: 40%; animation-delay: -3s; }
.f3 { width: 160px; height: 18px; background: var(--ink); top: 18%; right: 46%; animation-delay: -6s; }

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
.stat:nth-child(2) { border-color: var(--yellow); } .stat:nth-child(3) { border-color: var(--blue); } .stat:nth-child(4) { border-color: var(--ink); }
.stat strong { display: block; font-size: 2.4rem; }
.feature { transition: transform .35s ease, box-shadow .35s ease; }
.feature:hover { transform: translateY(-6px) rotateX(4deg); box-shadow: 0 20px 40px -24px rgba(0,0,0,.45); }
.dot { display: inline-block; width: 18px; height: 18px; }
.dot.red { background: var(--red); } .dot.yellow { background: var(--yellow); } .dot.blue { background: var(--blue); } .dot.black { background: var(--ink); }
.timeline { display: grid; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); gap: 1.2rem; position: relative; }
.step { position: relative; padding: 1.4rem; border: 1px solid var(--line); border-radius: var(--radius); background: var(--card); }
.step-number { font-size: 3rem; font-weight: 800; color: transparent; -webkit-text-stroke: 1.5px var(--accent); }
.preview-strip { display: flex; gap: 1.2rem; overflow-x: auto; padding: 1rem .2rem 2rem; perspective: 1000px; }
.tilt { flex: 0 0 180px; transition: transform .45s cubic-bezier(.2,.8,.2,1); }
.tilt:hover { transform: rotateY(-14deg) rotateX(6deg) translateZ(30px) scale(1.04); }

/* ─── Vignettes et catalogue ──────────────────────────────────────────── */
.thumb { position: relative; width: 100%; overflow: hidden; box-shadow: 0 14px 30px -18px rgba(0,0,0,.6); }
.thumb span { position: absolute; }
.frame { padding: 10px; background: #221e1c; border-radius: 4px; }
.art-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr)); gap: 1.6rem; }
.art-card { display: flex; flex-direction: column; gap: .3rem; }
.art-card .frame { transition: transform .4s ease; }
.art-card:hover .frame { transform: translateY(-4px) rotate(-1deg); }
.page-intro { padding: 3rem 0 1.5rem; }
.viewer { position: relative; border-radius: var(--radius); overflow: hidden; border: 1px solid var(--line); }
.scene-gallery { height: min(72vh, 620px); background: linear-gradient(#1a1b22, #0b0b10); }
.room-legend { margin-top: 2rem; }

/* ─── Le grand pied de page ───────────────────────────────────────────── */
.site-footer { display: block; margin-top: 5rem; padding: 0; border: 0; background: #0d0e14; color: #ece9e1; }
.site-footer a { color: #ece9e1; }
.site-footer .muted { color: #9a98a3; }
.footer-band { display: grid; grid-template-columns: 3fr 1fr 2fr 1fr 4fr 1fr 2fr 2fr; height: 14px; }
.footer-band span { transform-origin: bottom; animation: band 3.2s ease-in-out infinite; }
.marquee { overflow: hidden; border-bottom: 1px solid #262833; padding: 1.1rem 0; mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent); }
.marquee-track { display: flex; width: max-content; animation: marquee 40s linear infinite; }
.marquee:hover .marquee-track { animation-play-state: paused; }
.marquee-item { padding: 0 2rem; white-space: nowrap; font-size: 1.05rem; }
.marquee-item strong { color: var(--yellow); }
.footer-grid { max-width: 1180px; margin: 0 auto; padding: 3.5rem 1.25rem 2rem; display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr; gap: 2.5rem; }
.footer-brand p { color: #b9b6c2; max-width: 22rem; line-height: 1.6; }
.footer-brand .logo { color: #fff; }
.footer-cta { margin-top: .6rem; animation: pulse 2.6s ease-in-out infinite; }
.footer-col h3 { font-size: .8rem; text-transform: uppercase; letter-spacing: .18em; color: #9a98a3; margin-bottom: 1rem; }
.footer-col ul { list-style: none; padding: 0; margin: 0; display: grid; gap: .55rem; }
.footer-col a { text-decoration: none; background: linear-gradient(var(--red), var(--red)) 0 100% / 0 2px no-repeat; transition: background-size .3s ease; }
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
@keyframes pulse { 0%, 100% { box-shadow: 0 0 0 0 rgba(200,32,38,.5); } 50% { box-shadow: 0 0 0 12px rgba(200,32,38,0); } }

@media (max-width: 820px) {
  .hero { grid-template-columns: 1fr; text-align: center; min-height: auto; }
  .hero .row { justify-content: center; }
  .lead { margin-inline: auto; }
  .footer-grid { grid-template-columns: 1fr 1fr; }
  .site-header nav { gap: .7rem; font-size: .9rem; }
}
@media (max-width: 520px) { .footer-grid { grid-template-columns: 1fr; } }

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation: none !important; transition: none !important; }
  .reveal, .hero-title span { opacity: 1; }
}
"#;
