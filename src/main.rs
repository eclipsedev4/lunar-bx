#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align2, Color32, FontId, Margin, Rect, RichText, Rounding, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const BG: Color32 = Color32::from_rgb(15, 17, 23);
const PANEL: Color32 = Color32::from_rgb(20, 23, 32);
const CARD: Color32 = Color32::from_rgb(28, 32, 44);
const LINE: Color32 = Color32::from_rgb(38, 43, 58);
const ACCENT: Color32 = Color32::from_rgb(110, 168, 255);
const TEXT: Color32 = Color32::from_rgb(236, 239, 247);
const MUTED: Color32 = Color32::from_rgb(138, 146, 170);

type Entry = (&'static str, u64, &'static str);

const TOP: &[Entry] = &[
    ("Grow a Garden", 126884695634066, "Simulator"),
    ("Blade Ball", 13772394625, "Fighting"),
    ("Pet Simulator 99", 8737899170, "Simulator"),
    ("Doors", 6516141723, "Horror"),
    ("Murder Mystery 2", 142823291, "Mystery"),
    ("Drive World", 10704789056, "Racing"),
    ("Arsenal", 286090429, "Shooter"),
    ("Super Golf!", 4468711919, "Sports"),
];
const POPULAR: &[Entry] = &[
    ("Steal a Brainrot", 109983668079237, "Funny"),
    ("Dress to Impress", 15101393044, "Roleplay"),
    ("Brookhaven RP", 4924922222, "Roleplay"),
    ("Jailbreak", 606849621, "Action"),
    ("Adopt Me!", 920587237, "Roleplay"),
    ("Bee Swarm Simulator", 1537690962, "Simulator"),
    ("99 Nights in the Forest", 79546208627805, "Survival"),
];
const RISING: &[Entry] = &[
    ("Blox Fruits", 2753915549, "Adventure"),
    ("Fisch", 16732694052, "Simulator"),
    ("The Strongest Battlegrounds", 10449761463, "Fighting"),
    ("Tower of Hell", 1962086868, "Obby"),
    ("Rivals", 17625359962, "Shooter"),
    ("Tower Defense Simulator", 3260590327, "Strategy"),
];
const SECTIONS: [(&str, &[Entry]); 3] = [("Top playing", TOP), ("Popular", POPULAR), ("Up and coming", RISING)];

#[derive(Clone, Serialize, Deserialize)]
struct Game {
    name: String,
    place_id: u64,
    #[serde(default)]
    genre: String,
}

fn game(e: &Entry) -> Game {
    Game { name: e.0.into(), place_id: e.1, genre: e.2.into() }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Config {
    saved: Vec<Game>,
    selected: Game,
    close_on_launch: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { saved: vec![], selected: game(&TOP[0]), close_on_launch: false }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("lunar-bx").join("config.json")
}

fn save_config(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, json);
    }
}

fn hsv(h: f32, s: f32, v: f32) -> Color32 {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
    let (r, g, b) = match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_color(name: &str, s: f32, v: f32) -> Color32 {
    let h = name.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
    hsv((h % 360) as f32 / 360.0, s, v)
}

/// Draws one game tile. Returns (tile clicked, save pill clicked).
fn tile(ui: &mut egui::Ui, g: &Game, active: bool, saved: bool) -> (bool, bool) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(172.0, 124.0), Sense::click());
    let pill = Rect::from_min_size(egui::pos2(rect.max.x - 62.0, rect.min.y + 8.0), Vec2::new(54.0, 22.0));
    let pill_resp = ui.interact(pill, resp.id.with("save"), Sense::click());
    let p = ui.painter();
    p.rect_filled(rect, 12.0, hue_color(&g.name, 0.5, 0.42));
    let letter = g.name.chars().next().unwrap_or('?').to_uppercase().to_string();
    p.text(rect.center() - Vec2::new(0.0, 12.0), Align2::CENTER_CENTER, letter, FontId::proportional(46.0), Color32::from_white_alpha(70));
    let strip = Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - 46.0), rect.max);
    p.rect_filled(strip, Rounding { nw: 0.0, ne: 0.0, sw: 12.0, se: 12.0 }, Color32::from_black_alpha(160));
    p.text(strip.min + Vec2::new(10.0, 8.0), Align2::LEFT_TOP, &g.name, FontId::proportional(14.0), TEXT);
    p.text(strip.min + Vec2::new(10.0, 27.0), Align2::LEFT_TOP, &g.genre, FontId::proportional(11.0), MUTED);
    if active {
        p.rect_stroke(rect, 12.0, Stroke::new(2.0, ACCENT));
    } else if resp.hovered() {
        p.rect_stroke(rect, 12.0, Stroke::new(1.0, Color32::from_white_alpha(90)));
    }
    p.rect_filled(pill, 11.0, if saved { ACCENT } else { Color32::from_black_alpha(140) });
    let label = if saved { "Saved" } else { "Save" };
    p.text(pill.center(), Align2::CENTER_CENTER, label, FontId::proportional(12.0), if saved { BG } else { TEXT });
    (resp.clicked(), pill_resp.clicked())
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Home,
    Library,
    Settings,
}

struct App {
    cfg: Config,
    tab: Tab,
    section: usize,
    new_name: String,
    new_id: String,
    status: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::dark();
        v.panel_fill = BG;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = ACCENT;
        v.extreme_bg_color = CARD;
        v.widgets.inactive.rounding = 8.0.into();
        v.widgets.hovered.rounding = 8.0.into();
        v.widgets.active.rounding = 8.0.into();
        cc.egui_ctx.set_visuals(v);
        let text = std::fs::read_to_string(config_path()).ok();
        let cfg = text.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Self { cfg, tab: Tab::Home, section: 0, new_name: String::new(), new_id: String::new(), status: String::new() }
    }

    fn launch(&mut self, ctx: &egui::Context) {
        let url = format!("roblox://experiences/start?placeId={}", self.cfg.selected.place_id);
        if open::that(&url).is_ok() {
            self.status = format!("Launching {}…", self.cfg.selected.name);
            if self.cfg.close_on_launch {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else {
            self.status = "Couldn't open Roblox. Check that it's installed and you're signed in.".into();
        }
    }

    fn grid(&mut self, ui: &mut egui::Ui, games: &[Game]) {
        let (mut pick, mut toggle) = (None, None);
        ui.spacing_mut().item_spacing = Vec2::splat(12.0);
        ui.horizontal_wrapped(|ui| {
            for g in games {
                let active = self.cfg.selected.place_id == g.place_id;
                let saved = self.cfg.saved.iter().any(|s| s.place_id == g.place_id);
                let (c, s) = tile(ui, g, active, saved);
                if c {
                    pick = Some(g.clone());
                }
                if s {
                    toggle = Some(g.clone());
                }
            }
        });
        if let Some(g) = pick {
            self.cfg.selected = g;
            self.status.clear();
            save_config(&self.cfg);
        }
        if let Some(g) = toggle {
            if let Some(i) = self.cfg.saved.iter().position(|s| s.place_id == g.place_id) {
                self.cfg.saved.remove(i);
            } else {
                self.cfg.saved.push(g);
            }
            save_config(&self.cfg);
        }
    }

    fn hero(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let g = self.cfg.selected.clone();
        egui::Frame::none()
            .fill(hue_color(&g.name, 0.55, 0.32))
            .rounding(16.0)
            .inner_margin(Margin::same(28.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Ready to play").size(13.0).color(Color32::from_white_alpha(170)));
                ui.label(RichText::new(&g.name).size(38.0).strong());
                ui.label(RichText::new(format!("{}   Place ID {}", g.genre, g.place_id)).color(Color32::from_white_alpha(170)));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    let b = egui::Button::new(RichText::new("Launch").size(18.0).strong().color(BG)).fill(ACCENT);
                    if ui.add_sized([190.0, 48.0], b).clicked() {
                        self.launch(ctx);
                    }
                    ui.label(RichText::new(&self.status).color(Color32::from_white_alpha(190)));
                });
            });
    }

    fn home(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.hero(ui, ctx);
        ui.add_space(22.0);
        ui.horizontal(|ui| {
            for (i, (label, _)) in SECTIONS.iter().enumerate() {
                let on = self.section == i;
                let b = egui::Button::new(RichText::new(*label).size(14.0).color(if on { BG } else { TEXT }))
                    .fill(if on { ACCENT } else { CARD })
                    .stroke(Stroke::new(1.0, if on { ACCENT } else { LINE }))
                    .rounding(16.0);
                if ui.add_sized([label.len() as f32 * 8.0 + 32.0, 32.0], b).clicked() {
                    self.section = i;
                }
            }
        });
        ui.add_space(12.0);
        let games: Vec<Game> = SECTIONS[self.section].1.iter().map(game).collect();
        self.grid(ui, &games);
    }

    fn library(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Library").size(30.0).strong());
        ui.label(RichText::new("Games you saved. Use Save on any game tile.").color(MUTED));
        ui.add_space(14.0);
        let games = self.cfg.saved.clone();
        if games.is_empty() {
            ui.label(RichText::new("Nothing saved yet. Save a game from Home or add one below.").color(MUTED));
        } else {
            self.grid(ui, &games);
        }
        ui.add_space(24.0);
        ui.label(RichText::new("Add by place ID").size(18.0).strong());
        ui.label(RichText::new("The place ID is the number in the game's Roblox URL.").color(MUTED));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_sized([220.0, 32.0], egui::TextEdit::singleline(&mut self.new_name).hint_text("Game name"));
            ui.add_sized([170.0, 32.0], egui::TextEdit::singleline(&mut self.new_id).hint_text("Place ID"));
            let b = egui::Button::new(RichText::new("Add game").color(BG)).fill(ACCENT);
            if ui.add_sized([100.0, 32.0], b).clicked() {
                if let Ok(id) = self.new_id.trim().parse::<u64>() {
                    let n = self.new_name.trim();
                    let name = if n.is_empty() { format!("Game {id}") } else { n.to_string() };
                    self.cfg.saved.push(Game { name, place_id: id, genre: "Custom".into() });
                    self.new_name.clear();
                    self.new_id.clear();
                    save_config(&self.cfg);
                }
            }
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Settings").size(30.0).strong());
        ui.add_space(14.0);
        if ui.checkbox(&mut self.cfg.close_on_launch, "Close the launcher after launching").changed() {
            save_config(&self.cfg);
        }
        ui.add_space(16.0);
        ui.label(RichText::new("Config file").strong());
        ui.label(RichText::new(config_path().display().to_string()).color(MUTED));
    }

    fn nav(&mut self, ui: &mut egui::Ui, label: &str, tab: Tab) {
        let on = self.tab == tab;
        let b = egui::Button::new(RichText::new(label).size(15.0).color(if on { TEXT } else { MUTED }))
            .fill(if on { CARD } else { Color32::TRANSPARENT })
            .stroke(Stroke::new(1.0, if on { LINE } else { Color32::TRANSPARENT }));
        if ui.add_sized([168.0, 40.0], b).clicked() {
            self.tab = tab;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        egui::SidePanel::left("nav")
            .exact_width(204.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(PANEL).inner_margin(Margin::same(18.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
                    let p = ui.painter();
                    p.circle_filled(r.center(), 12.0, TEXT);
                    p.circle_filled(r.center() + Vec2::new(6.0, -4.0), 11.0, PANEL);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new("LUNAR BX").size(16.0).strong());
                        ui.label(RichText::new("for Roblox").size(11.0).color(MUTED));
                    });
                });
                ui.add_space(26.0);
                self.nav(ui, "Home", Tab::Home);
                self.nav(ui, "Library", Tab::Library);
                self.nav(ui, "Settings", Tab::Settings);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new("Not affiliated with Roblox").size(11.0).color(MUTED));
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(Margin::same(28.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| match self.tab {
                    Tab::Home => self.home(ui, ctx),
                    Tab::Library => self.library(ui),
                    Tab::Settings => self.settings(ui),
                });
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 680.0])
            .with_min_inner_size([820.0, 520.0])
            .with_title("Lunar BX"),
        ..Default::default()
    };
    eframe::run_native("Lunar BX", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
