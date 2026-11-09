//! L'interface du musée, en egui (avec eframe), dessinée dans un canvas transparent posé
//! sur la scène 3D : la barre de visite, le cartel, la minicarte, la liste des œuvres et
//! l'éditeur. Elle lit et écrit les commandes partagées avec la scène ([`TourControl`]).
//!
//! Les clics qui ne touchent aucun panneau traversent l'interface : ils vont à la scène
//! (plein écran, visite libre à la souris ou au doigt).

use std::cell::Cell;
use std::rc::Rc;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Shape, Stroke, Vec2,
};

use crate::artworks::{Exhibition, Side, Work};
use crate::editor::{self, ArtEdit, SculptEdit};
use crate::gallery::{Mode, SharedControl};
use crate::museum;
use crate::texts::SharedTexts;
use crate::tour::Focus;

/// Fait télécharger un texte (le nom du fichier, puis son contenu).
pub type Download = Rc<dyn Fn(&str, &str)>;

/// Ce que l'interface demande à la page (plein écran, image HD, fichiers).
pub struct Actions {
    pub enter: Rc<dyn Fn()>,
    pub exit: Rc<dyn Fn()>,
    pub is_full: Rc<dyn Fn() -> bool>,
    pub open_hd: Rc<dyn Fn()>,
    pub download: Download,
    pub copy: Rc<dyn Fn(&str)>,
    pub open: Rc<dyn Fn(&str)>,
}

/// L'adresse du dépôt, pour publier le fichier modifié.
const REPOSITORY: &str = "https://github.com/maxlestage/ARTEON";

pub struct MuseumUi {
    ex: &'static Exhibition,
    control: SharedControl,
    actions: Actions,
    /// L'interface a-t-elle le pointeur (la scène ne doit alors pas réagir) ?
    busy: Rc<Cell<bool>>,
    accent: Color32,
    show_map: bool,
    show_list: bool,
    editing: bool,
    selected: Option<Focus>,
    notice: Option<String>,
    /// La notice complète de l'œuvre regardée est-elle ouverte ?
    reading: bool,
    /// La présentation du bâtiment est-elle ouverte ?
    building: bool,
    /// Les textes longs (notices, biographies, bâtiment, fichier source), téléchargés.
    texts: SharedTexts,
}

impl MuseumUi {
    pub fn new(
        ex: &'static Exhibition,
        control: SharedControl,
        actions: Actions,
        busy: Rc<Cell<bool>>,
        texts: SharedTexts,
    ) -> MuseumUi {
        let [r, g, b] = ex.accent;
        MuseumUi {
            ex,
            control,
            actions,
            busy,
            accent: Color32::from_rgb(r, g, b),
            show_map: false,
            show_list: false,
            editing: false,
            selected: None,
            notice: None,
            reading: false,
            building: false,
            texts,
        }
    }

    fn frame(&self) -> egui::Frame {
        egui::Frame::new()
            .fill(Color32::from_rgba_unmultiplied(12, 16, 32, 205))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(egui::Margin::same(10))
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            ))
    }

    fn toggle(&self, ui: &mut egui::Ui, on: bool, text: &str) -> egui::Response {
        let fill = if on {
            self.accent
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 22)
        };
        ui.add(
            egui::Button::new(RichText::new(text).color(Color32::WHITE))
                .fill(fill)
                .corner_radius(CornerRadius::same(16)),
        )
    }

    /// La barre de visite, en haut à gauche.
    fn toolbar(&mut self, ctx: &egui::Context) {
        let (mode, paused) = {
            let c = self.control.borrow();
            (c.mode, c.paused)
        };
        egui::Area::new(egui::Id::unique("barre")).anchor(Align2::LEFT_TOP, [12.0, 12.0]).show(ctx, |ui| {
            self.frame().show(ui, |ui| {
                ui.set_max_width(ctx.content_rect().width() - 40.0);
                ui.horizontal_wrapped(|ui| {
                    let guided = mode == Mode::Guided;
                    if self.toggle(ui, guided, "🎬 Guidée").on_hover_text("La caméra vous emmène d'œuvre en œuvre").clicked() {
                        self.set_mode(Mode::Guided);
                    }
                    if self.toggle(ui, !guided, "🚶 Libre").on_hover_text("Se promener soi-même : ZQSD/WASD, flèches, souris ; au doigt, deux joysticks").clicked() {
                        self.set_mode(Mode::Free);
                    }
                    ui.separator();
                    if self.toggle(ui, false, "‹").on_hover_text("Œuvre précédente").clicked() {
                        self.step(-1);
                    }
                    if guided && self.toggle(ui, paused, if paused { "▶" } else { "❚❚" }).on_hover_text("Pause").clicked() {
                        let mut c = self.control.borrow_mut();
                        c.paused = !c.paused;
                    }
                    if self.toggle(ui, false, "›").on_hover_text("Œuvre suivante").clicked() {
                        self.step(1);
                    }
                    ui.separator();
                    if self.toggle(ui, self.show_map, "🗺 Plan").clicked() {
                        self.show_map = !self.show_map;
                    }
                    if self.toggle(ui, self.show_list, "☰ Œuvres").clicked() {
                        self.show_list = !self.show_list;
                    }
                    if self.toggle(ui, self.building, "🏛 Le musée").on_hover_text("Le bâtiment et ses salles").clicked() {
                        self.building = !self.building;
                    }
                    if self.toggle(ui, self.editing, "✏ Éditer").on_hover_text("Déplacer les œuvres, puis télécharger le fichier de l'exposition").clicked() {
                        self.editing = !self.editing;
                        if self.editing {
                            self.set_mode(Mode::Free);
                            self.selected = self.selected.or(self.control.borrow().current);
                        }
                    }
                    ui.separator();
                    if (self.actions.is_full)() {
                        if self.toggle(ui, false, "✕").on_hover_text("Quitter le plein écran").clicked() {
                            (self.actions.exit)();
                        }
                    } else if self.toggle(ui, false, "⛶").on_hover_text("Plein écran").clicked() {
                        (self.actions.enter)();
                    }
                });
            });
        });
    }

    fn set_mode(&self, mode: Mode) {
        let mut c = self.control.borrow_mut();
        c.mode = mode;
        (c.forward, c.strafe, c.spin) = (0.0, 0.0, 0.0);
    }

    /// L'œuvre précédente ou suivante, dans l'ordre de la visite.
    fn step(&self, step: i32) {
        let mut c = self.control.borrow_mut();
        if c.mode == Mode::Free {
            let order: Vec<Focus> = crate::tour::stops(self.ex)
                .iter()
                .filter_map(|s| s.focus)
                .collect();
            if !order.is_empty() {
                let at = c.current.and_then(|f| order.iter().position(|o| *o == f));
                let k = at.map_or(0, |i| {
                    (i as i32 + step).rem_euclid(order.len() as i32) as usize
                });
                c.go_to = Some(order[k]);
            }
        } else {
            c.jump += step;
        }
    }

    /// Le cartel de l'œuvre regardée, en bas à gauche.
    fn caption(&mut self, ctx: &egui::Context) {
        let Some(focus) = self.control.borrow().current else {
            return;
        };
        let ex = self.ex;
        let (title, meta, note, technique) = match focus {
            Focus::Art(i) => {
                let a = &ex.artworks[i];
                (
                    a.title,
                    format!("{} · {}", a.artist_in(ex), a.year),
                    a.note,
                    format!("{} — {}", a.technique, a.collection),
                )
            }
            Focus::Sculpture(i) => {
                let s = &ex.sculptures[i];
                (
                    s.title,
                    format!("{} · {}", s.artist, s.year),
                    s.note,
                    format!("{} — {}", s.technique, s.collection),
                )
            }
        };
        let width = (ctx.content_rect().width() - 40.0).min(380.0);
        egui::Area::new(egui::Id::unique("cartel"))
            .anchor(Align2::LEFT_BOTTOM, [12.0, -14.0])
            .show(ctx, |ui| {
                self.frame().show(ui, |ui| {
                    ui.set_max_width(width);
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .size(16.0)
                            .color(Color32::WHITE),
                    );
                    ui.label(RichText::new(meta).color(Color32::from_gray(210)));
                    if ctx.content_rect().height() > 520.0 {
                        ui.label(
                            RichText::new(technique)
                                .small()
                                .color(Color32::from_gray(170)),
                        );
                        ui.label(RichText::new(note).color(Color32::from_gray(225)));
                    }
                    ui.horizontal_wrapped(|ui| {
                        if self.toggle(ui, self.reading, "📖 Tout savoir").clicked() {
                            self.reading = !self.reading;
                            if self.reading {
                                // La visite attend le lecteur devant l'œuvre.
                                self.control.borrow_mut().paused = true;
                            }
                        }
                        if matches!(focus, Focus::Art(_))
                            && self.toggle(ui, false, "Voir en HD").clicked()
                        {
                            (self.actions.open_hd)();
                        }
                    });
                });
            });
    }

    /// La notice complète de l'œuvre regardée : ce que l'on voit, ce qu'elle évoque, son
    /// histoire, la manière, les détails, l'anecdote et l'artiste.
    fn reader(&mut self, ctx: &egui::Context) {
        let ex = self.ex;
        let Some(focus) = self.control.borrow().current else {
            return;
        };
        let work = match focus {
            Focus::Art(i) => Work::Art(&ex.artworks[i]),
            Focus::Sculpture(i) => Work::Sculpture(&ex.sculptures[i]),
        };
        let (year, technique, collection, note) = match work {
            Work::Art(a) => (a.year, a.technique, a.collection, a.note),
            Work::Sculpture(s) => (s.year, s.technique, s.collection, s.note),
        };
        let artist = work.artist(ex);
        let themes = work.notice().themes;
        let texts = self.texts.clone();
        let screen = ctx.content_rect();
        let width = (screen.width() - 24.0).min(520.0);
        let mut open = true;
        let text = |t: &str| {
            RichText::new(t.to_owned())
                .size(14.5)
                .color(Color32::from_gray(230))
        };
        let heading = |t: &str| {
            RichText::new(t.to_owned())
                .strong()
                .size(15.5)
                .color(self.accent.gamma_multiply(1.6))
        };
        egui::Window::new("Tout savoir")
            .id(egui::Id::unique("notice"))
            .open(&mut open)
            .collapsible(false)
            .anchor(Align2::RIGHT_TOP, [-12.0, 64.0])
            .fixed_size([width, (screen.height() - 90.0).max(200.0)])
            .frame(self.frame())
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.set_max_width(width - 16.0);
                        ui.label(
                            RichText::new(work.title())
                                .strong()
                                .size(19.0)
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            RichText::new(format!("{artist} · {year}"))
                                .size(15.0)
                                .color(Color32::from_gray(215)),
                        );
                        if !technique.is_empty() || !collection.is_empty() {
                            ui.label(
                                RichText::new(format!("{technique} — {collection}"))
                                    .small()
                                    .color(Color32::from_gray(170)),
                            );
                        }
                        ui.add_space(6.0);
                        ui.label(text(note).italics());
                        if !themes.is_empty() {
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                for t in themes {
                                    egui::Frame::new()
                                        .fill(self.accent.gamma_multiply(0.55))
                                        .corner_radius(CornerRadius::same(10))
                                        .inner_margin(egui::Margin::symmetric(8, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(*t).small().color(Color32::WHITE),
                                            )
                                        });
                                }
                            });
                        }
                        let texts = texts.borrow();
                        let Some(texts) = texts.as_ref() else {
                            ui.add_space(10.0);
                            ui.spinner();
                            ui.label(text("Chargement de la notice…"));
                            return;
                        };
                        let empty = crate::texts::WorkText::default();
                        let notice = texts.works.get(work.slug()).unwrap_or(&empty);
                        for (title, body) in notice.sections() {
                            ui.add_space(10.0);
                            ui.label(heading(title));
                            ui.label(text(body));
                        }
                        if !notice.details.is_empty() {
                            ui.add_space(10.0);
                            ui.label(heading("À regarder de près"));
                            for d in &notice.details {
                                ui.label(text(&format!("•  {d}")));
                            }
                        }
                        if !notice.anecdote.is_empty() {
                            ui.add_space(10.0);
                            egui::Frame::new()
                                .fill(Color32::from_white_alpha(14))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.label(heading("Le saviez-vous ?"));
                                    ui.label(text(&notice.anecdote));
                                });
                        }
                        if let Some(about) = texts.artists.get(artist) {
                            ui.add_space(10.0);
                            egui::CollapsingHeader::new(heading(&format!("L'artiste : {artist}")))
                                .id_salt(("artiste", artist))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(&about.life)
                                            .small()
                                            .color(Color32::from_gray(180)),
                                    );
                                    ui.label(text(&about.biography));
                                    if !about.style.is_empty() {
                                        ui.add_space(6.0);
                                        ui.label(heading("Sa manière"));
                                        ui.label(text(&about.style));
                                    }
                                });
                        }
                        ui.add_space(10.0);
                        if ui.button("Sa page sur le site ↗").clicked() {
                            let url = format!(
                                "{}/{}/oeuvres/{}",
                                crate::viewer::base_path(ex.slug),
                                ex.slug,
                                work.slug()
                            );
                            (self.actions.open)(&url);
                        }
                    });
            });
        if !open {
            self.reading = false;
        }
    }

    /// Le bâtiment : le vrai musée dont il s'inspire, ce qu'il en garde, et ses salles.
    fn building_window(&mut self, ctx: &egui::Context) {
        let ex = self.ex;
        let screen = ctx.content_rect();
        let width = (screen.width() - 24.0).min(500.0);
        let mut open = true;
        let text = |t: &str| {
            RichText::new(t.to_owned())
                .size(14.5)
                .color(Color32::from_gray(230))
        };
        let heading = |t: &str| {
            RichText::new(t.to_owned())
                .strong()
                .size(15.5)
                .color(self.accent.gamma_multiply(1.6))
        };
        egui::Window::new("Le musée")
            .id(egui::Id::unique("batiment"))
            .open(&mut open)
            .collapsible(false)
            .anchor(Align2::RIGHT_TOP, [-12.0, 64.0])
            .fixed_size([width, (screen.height() - 90.0).max(200.0)])
            .frame(self.frame())
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.set_max_width(width - 16.0);
                        match ex.museum {
                            Some(m) => {
                                ui.label(
                                    RichText::new("Inspiré de")
                                        .small()
                                        .color(Color32::from_gray(170)),
                                );
                                ui.label(
                                    RichText::new(m.name)
                                        .strong()
                                        .size(19.0)
                                        .color(Color32::WHITE),
                                );
                                let facts: Vec<&str> = [m.city, m.architect, m.built]
                                    .into_iter()
                                    .filter(|t| !t.is_empty())
                                    .collect();
                                if !facts.is_empty() {
                                    ui.label(
                                        RichText::new(facts.join(" · "))
                                            .color(Color32::from_gray(200)),
                                    );
                                }
                                match self.texts.borrow().as_ref() {
                                    Some(t) => {
                                        if !t.presentation.is_empty() {
                                            ui.add_space(8.0);
                                            ui.label(text(&t.presentation));
                                        }
                                        if !t.here.is_empty() {
                                            ui.add_space(10.0);
                                            ui.label(heading("Dans ARTEON"));
                                            ui.label(text(&t.here));
                                        }
                                    }
                                    None => {
                                        ui.spinner();
                                    }
                                }
                            }
                            None => {
                                ui.label(
                                    RichText::new(ex.title)
                                        .strong()
                                        .size(19.0)
                                        .color(Color32::WHITE),
                                );
                            }
                        }
                        ui.add_space(10.0);
                        ui.label(heading("Les salles"));
                        for (n, (i, room)) in ex.galleries().enumerate() {
                            let works = ex.works_in_room(i).len() + ex.sculptures_in_room(i).len();
                            let line = format!("{}. {} · {works} œuvre(s)", n + 1, room.name);
                            if ui
                                .add(egui::Label::new(text(&line)).sense(Sense::click()))
                                .on_hover_text("S'y rendre")
                                .clicked()
                            {
                                self.set_mode(Mode::Free);
                                self.control.borrow_mut().teleport = Some((room.x, room.z));
                            }
                        }
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(
                                "Le plan (bouton 🗺) montre les salles, les portes et les œuvres.",
                            )
                            .small()
                            .color(Color32::from_gray(170)),
                        );
                    });
            });
        if !open {
            self.building = false;
        }
    }

    /// La minicarte : les salles, les portes, les œuvres et le visiteur. Cliquer sur une
    /// œuvre y emmène ; en visite libre, cliquer sur le sol y transporte.
    fn map(&mut self, ctx: &egui::Context) {
        let ex = self.ex;
        let (x0, x1, z0, z1) = ex.rooms.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(a, b, c, d), r| {
                (
                    a.min(r.x - r.width / 2.0),
                    b.max(r.x + r.width / 2.0),
                    c.min(r.z - r.depth / 2.0),
                    d.max(r.z + r.depth / 2.0),
                )
            },
        );
        let side = (ctx.content_rect().width() * 0.42).clamp(170.0, 300.0);
        let scale = side / (x1 - x0).max(z1 - z0).max(1.0);
        let size = Vec2::new((x1 - x0) * scale, (z1 - z0) * scale);
        egui::Area::new(egui::Id::unique("plan"))
            .anchor(Align2::RIGHT_TOP, [-12.0, 64.0])
            .show(ctx, |ui| {
                self.frame().show(ui, |ui| {
                    ui.label(
                        RichText::new("Plan du musée")
                            .strong()
                            .color(Color32::WHITE),
                    );
                    let (response, painter) = ui.allocate_painter(size, Sense::click());
                    let origin = response.rect.min;
                    let at = |x: f32, z: f32| {
                        Pos2::new(origin.x + (x - x0) * scale, origin.y + (z - z0) * scale)
                    };
                    let edits = self.control.borrow().edits.clone();
                    for room in ex.rooms {
                        let rect = Rect::from_min_max(
                            at(room.x - room.width / 2.0, room.z - room.depth / 2.0),
                            at(room.x + room.width / 2.0, room.z + room.depth / 2.0),
                        );
                        let fill = if room.corridor {
                            Color32::from_gray(52)
                        } else {
                            Color32::from_gray(72)
                        };
                        painter.rect_filled(rect, CornerRadius::ZERO, fill);
                        painter.rect_stroke(
                            rect,
                            CornerRadius::ZERO,
                            Stroke::new(1.5, Color32::from_gray(200)),
                            egui::StrokeKind::Inside,
                        );
                        for d in room.doors {
                            let (a, b) = (
                                room.point_on(d.side, d.position - d.width / 2.0),
                                room.point_on(d.side, d.position + d.width / 2.0),
                            );
                            painter
                                .line_segment([at(a.0, a.1), at(b.0, b.1)], Stroke::new(3.0, fill));
                        }
                    }
                    let current = self.control.borrow().current;
                    for i in 0..ex.artworks.len() {
                        let art = editor::edited_art(ex, &edits, i);
                        let room = &ex.rooms[art.hook.room];
                        let (a, b) = (
                            room.point_on(art.hook.side, art.hook.position - art.width / 2.0),
                            room.point_on(art.hook.side, art.hook.position + art.width / 2.0),
                        );
                        let on =
                            current == Some(Focus::Art(i)) || self.selected == Some(Focus::Art(i));
                        painter.line_segment(
                            [at(a.0, a.1), at(b.0, b.1)],
                            Stroke::new(
                                if on { 5.0 } else { 3.0 },
                                if on { Color32::WHITE } else { self.accent },
                            ),
                        );
                    }
                    for i in 0..ex.sculptures.len() {
                        let s = editor::edited_sculpture(ex, &edits, i);
                        let on = current == Some(Focus::Sculpture(i))
                            || self.selected == Some(Focus::Sculpture(i));
                        painter.circle_filled(
                            at(s.x, s.z),
                            (s.radius * scale).max(3.0),
                            if on {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(214, 208, 196)
                            },
                        );
                    }
                    let [vx, vz, yaw] = self.control.borrow().position;
                    let tip = at(vx + yaw.sin() * 1.4, vz + yaw.cos() * 1.4);
                    let left = at(vx + (yaw + 2.5).sin() * 0.8, vz + (yaw + 2.5).cos() * 0.8);
                    let right = at(vx + (yaw - 2.5).sin() * 0.8, vz + (yaw - 2.5).cos() * 0.8);
                    painter.add(Shape::convex_polygon(
                        vec![tip, left, right],
                        Color32::from_rgb(242, 183, 5),
                        Stroke::NONE,
                    ));
                    if let Some(pos) = response
                        .interact_pointer_pos()
                        .filter(|_| response.clicked())
                    {
                        let (x, z) = (
                            x0 + (pos.x - origin.x) / scale,
                            z0 + (pos.y - origin.y) / scale,
                        );
                        self.map_click(x, z);
                    }
                });
            });
    }

    fn map_click(&mut self, x: f32, z: f32) {
        let ex = self.ex;
        let edits = self.control.borrow().edits.clone();
        let mut best: Option<(f32, Focus)> = None;
        for i in 0..ex.artworks.len() {
            let a = editor::edited_art(ex, &edits, i);
            let c = museum::on_wall(ex, a.hook, 0.0, 0.0, 0.0).0;
            let d = (c[0] - x).hypot(c[2] - z);
            if d < 1.6 && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, Focus::Art(i)));
            }
        }
        for i in 0..ex.sculptures.len() {
            let s = editor::edited_sculpture(ex, &edits, i);
            let d = (s.x - x).hypot(s.z - z);
            if d < s.radius + 0.8 && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, Focus::Sculpture(i)));
            }
        }
        let mut c = self.control.borrow_mut();
        match best {
            Some((_, focus)) => {
                if self.editing {
                    self.selected = Some(focus);
                } else {
                    c.go_to = Some(focus);
                }
            }
            None if c.mode == Mode::Free && museum::room_at(ex, x, z).is_some() => {
                c.teleport = Some((x, z))
            }
            None => {}
        }
    }

    /// La liste des œuvres, salle par salle : cliquer y emmène.
    fn list(&mut self, ctx: &egui::Context) {
        let ex = self.ex;
        let height = (ctx.content_rect().height() - 180.0).max(160.0);
        egui::Area::new(egui::Id::unique("liste"))
            .anchor(Align2::RIGHT_BOTTOM, [-12.0, -14.0])
            .show(ctx, |ui| {
                self.frame().show(ui, |ui| {
                    ui.set_max_width(300.0);
                    ui.label(RichText::new("Les œuvres").strong().color(Color32::WHITE));
                    egui::ScrollArea::vertical()
                        .max_height(height)
                        .show(ui, |ui| {
                            let current = self.control.borrow().current;
                            for (r, room) in ex.galleries() {
                                ui.label(
                                    RichText::new(room.name)
                                        .small()
                                        .color(Color32::from_gray(170)),
                                );
                                for art in ex.works_in_room(r) {
                                    let i = ex
                                        .artworks
                                        .iter()
                                        .position(|a| std::ptr::eq(a, art))
                                        .unwrap_or(0);
                                    if ui
                                        .selectable_label(
                                            current == Some(Focus::Art(i)),
                                            format!("🖼 {}", art.title),
                                        )
                                        .clicked()
                                    {
                                        self.pick(Focus::Art(i));
                                    }
                                }
                                for (i, s) in ex
                                    .sculptures
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, s)| s.room == r)
                                {
                                    if ui
                                        .selectable_label(
                                            current == Some(Focus::Sculpture(i)),
                                            format!("🗿 {}", s.title),
                                        )
                                        .clicked()
                                    {
                                        self.pick(Focus::Sculpture(i));
                                    }
                                }
                            }
                        });
                });
            });
    }

    fn pick(&mut self, focus: Focus) {
        if self.editing {
            self.selected = Some(focus);
        }
        self.control.borrow_mut().go_to = Some(focus);
    }

    /// L'éditeur : la place de l'œuvre choisie, et le fichier à publier.
    fn editor(&mut self, ctx: &egui::Context) {
        let ex = self.ex;
        let mut open = self.editing;
        egui::Window::new("Éditeur du musée")
            .open(&mut open)
            .default_pos([12.0, 70.0])
            .default_width(320.0)
            .frame(self.frame())
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(230));
                let label = |f: Option<Focus>| match f {
                    Some(Focus::Art(i)) => format!("🖼 {}", ex.artworks[i].title),
                    Some(Focus::Sculpture(i)) => format!("🗿 {}", ex.sculptures[i].title),
                    None => "Choisir une œuvre…".to_owned(),
                };
                egui::ComboBox::from_id_salt("oeuvre").width(290.0).selected_text(label(self.selected)).show_ui(ui, |ui| {
                    for i in 0..ex.artworks.len() {
                        ui.selectable_value(&mut self.selected, Some(Focus::Art(i)), label(Some(Focus::Art(i))));
                    }
                    for i in 0..ex.sculptures.len() {
                        ui.selectable_value(&mut self.selected, Some(Focus::Sculpture(i)), label(Some(Focus::Sculpture(i))));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("L'œuvre regardée").clicked() {
                        self.selected = self.control.borrow().current;
                    }
                    if let Some(focus) = self.selected
                        && ui.button("Y aller").clicked()
                    {
                        self.control.borrow_mut().go_to = Some(focus);
                    }
                });
                ui.separator();
                match self.selected {
                    Some(Focus::Art(i)) => self.edit_art(ui, i),
                    Some(Focus::Sculpture(i)) => self.edit_sculpture(ui, i),
                    None => {
                        ui.label("Choisissez une œuvre dans la liste, sur le plan, ou regardez-la puis « L'œuvre regardée ».");
                    }
                }
                ui.separator();
                self.publish(ui);
            });
        if !open {
            self.editing = false;
        }
    }

    fn edit_art(&mut self, ui: &mut egui::Ui, i: usize) {
        let ex = self.ex;
        let mut e: ArtEdit = self.control.borrow().edits.art(ex, i);
        let before = e;
        egui::ComboBox::from_label("Salle")
            .selected_text(ex.rooms[e.room].name)
            .show_ui(ui, |ui| {
                for (r, room) in ex.galleries() {
                    ui.selectable_value(&mut e.room, r, room.name);
                }
            });
        ui.horizontal(|ui| {
            ui.label("Mur");
            for side in Side::ALL {
                ui.selectable_value(&mut e.side, side, side.french());
            }
        });
        let room = &ex.rooms[e.room];
        let half = room.wall_length(e.side) / 2.0;
        e.position = e.position.clamp(-half, half);
        ui.add(
            egui::Slider::new(&mut e.position, -half..=half)
                .step_by(0.05)
                .text("position (m)"),
        );
        ui.add(
            egui::Slider::new(
                &mut e.width,
                0.3..=(room.wall_length(e.side) - 1.0).max(0.4),
            )
            .step_by(0.05)
            .text("largeur (m)"),
        );
        if e != before {
            let mut c = self.control.borrow_mut();
            c.edits.arts.insert(i, e);
            c.edits.version += 1;
        }
        self.problems(
            ui,
            editor::art_problems(ex, &self.control.borrow().edits, i),
        );
    }

    fn edit_sculpture(&mut self, ui: &mut egui::Ui, i: usize) {
        let ex = self.ex;
        let mut e: SculptEdit = self.control.borrow().edits.sculpture(ex, i);
        let before = e;
        egui::ComboBox::from_label("Salle")
            .selected_text(ex.rooms[e.room].name)
            .show_ui(ui, |ui| {
                for (r, room) in ex.galleries() {
                    ui.selectable_value(&mut e.room, r, room.name);
                }
            });
        let room = &ex.rooms[e.room];
        let (hw, hd) = (room.width / 2.0, room.depth / 2.0);
        e.x = e.x.clamp(-hw, hw);
        e.z = e.z.clamp(-hd, hd);
        ui.add(
            egui::Slider::new(&mut e.x, -hw..=hw)
                .step_by(0.05)
                .text("x, vers l'est (m)"),
        );
        ui.add(
            egui::Slider::new(&mut e.z, -hd..=hd)
                .step_by(0.05)
                .text("z, vers le sud (m)"),
        );
        ui.add(
            egui::Slider::new(&mut e.rotation, -180.0..=180.0)
                .step_by(5.0)
                .text("rotation (°)"),
        );
        ui.add(
            egui::Slider::new(&mut e.height, 0.1..=(room.height - 0.5).max(0.2))
                .step_by(0.01)
                .text("hauteur (m)"),
        );
        ui.add(
            egui::Slider::new(&mut e.plinth, 0.0..=1.5)
                .step_by(0.05)
                .text("socle (m)"),
        );
        if e != before {
            let mut c = self.control.borrow_mut();
            c.edits.sculptures.insert(i, e);
            c.edits.version += 1;
        }
        self.problems(
            ui,
            editor::sculpture_problems(ex, &self.control.borrow().edits, i),
        );
    }

    fn problems(&self, ui: &mut egui::Ui, problems: Vec<String>) {
        if problems.is_empty() {
            ui.label(
                RichText::new("✔ Cette place est valide").color(Color32::from_rgb(120, 200, 140)),
            );
        }
        for p in problems {
            ui.label(
                RichText::new(format!("⚠ {p} : la publication serait refusée"))
                    .color(Color32::from_rgb(240, 150, 110)),
            );
        }
    }

    /// Télécharger, copier ou publier le fichier modifié.
    fn publish(&mut self, ui: &mut egui::Ui) {
        let ex = self.ex;
        let count = {
            let c = self.control.borrow();
            c.edits.arts.len() + c.edits.sculptures.len()
        };
        ui.label(format!("{count} œuvre(s) déplacée(s). Le fichier expositions/{}.toml garde tout le reste tel quel.", ex.slug));
        ui.add_enabled_ui(count > 0, |ui| {
            ui.horizontal_wrapped(|ui| {
                let file = format!("{}.toml", ex.slug);
                let source = self.texts.borrow().as_ref().map(|t| t.source.clone());
                let Some(source) = source.filter(|s| !s.is_empty()) else {
                    ui.spinner();
                    ui.label("Le fichier de l'exposition se charge…");
                    return;
                };
                if ui.button("⬇ Télécharger").clicked() {
                    let text = editor::patched_toml(&source, ex, &self.control.borrow().edits);
                    (self.actions.download)(&file, &text);
                    self.notice = Some(format!(
                        "{file} téléchargé : remplacez expositions/{file} dans le dépôt."
                    ));
                }
                if ui.button("📋 Copier").clicked() {
                    let text = editor::patched_toml(&source, ex, &self.control.borrow().edits);
                    (self.actions.copy)(&text);
                    self.notice = Some("Copié : collez-le dans l'éditeur de GitHub.".to_owned());
                }
                if ui.button("Publier sur GitHub ↗").clicked() {
                    let text = editor::patched_toml(&source, ex, &self.control.borrow().edits);
                    (self.actions.copy)(&text);
                    (self.actions.open)(&format!("{REPOSITORY}/edit/master/expositions/{file}"));
                    self.notice = Some(
                        "Fichier copié : sur GitHub, remplacez tout le texte par le vôtre et validez."
                            .to_owned(),
                    );
                }
                if ui.button("Annuler tout").clicked() {
                    let mut c = self.control.borrow_mut();
                    c.edits.arts.clear();
                    c.edits.sculptures.clear();
                    c.edits.version += 1;
                }
            });
        });
        if let Some(n) = &self.notice {
            ui.label(RichText::new(n).small().color(Color32::from_gray(190)));
        }
    }
}

impl eframe::App for MuseumUi {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.content_rect().width() < 560.0 {
            ctx.set_zoom_factor(1.05);
        }
        self.toolbar(&ctx);
        if self.show_map {
            self.map(&ctx);
        }
        if self.show_list {
            self.list(&ctx);
        } else {
            self.caption(&ctx);
        }
        if self.editing {
            self.editor(&ctx);
        }
        if self.reading {
            self.reader(&ctx);
        }
        if self.building {
            self.building_window(&ctx);
        }
        self.busy
            .set(ctx.is_pointer_over_egui() || ctx.egui_wants_pointer_input());
        // Le cartel et la minicarte suivent la scène : redessiner régulièrement.
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
