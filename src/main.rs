#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, Margin, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

// Palette: deep navy base, lighter navy panels, Lunar-ish sky blue accent.
const BG: Color32 = Color32::from_rgb(11, 14, 28);
const PANEL: Color32 = Color32::from_rgb(18, 22, 42);
const CARD: Color32 = Color32::from_rgb(26, 32, 58);
const CARD_ACTIVE: Color32 = Color32::from_rgb(34, 48, 92);
const ACCENT: Color32 = Color32::from_rgb(86, 156, 255);
const TEXT: Color32 = Color32::from_rgb(232, 236, 248);
const MUTED: Color32 = Color32::from_rgb(133, 142, 172);
const DANGER: Color32 = Color32::from_rgb(255, 107, 120);

#[derive(Clone, Serialize, Deserialize)]
struct Game {
    name: String,
    place_id: u64,
}

#[derive(Serialize, Deserialize)]
struct Config {
    games: Vec<Game>,
    selected: usize,
    close_on_launch: bool,
}

impl Default for Config {
    fn default() -> Self {
        let g = |name: &str, place_id| Game { name: name.into(), place_id };
        Self {
            games: vec![
                g("Adopt Me!", 920587237),
                g("Blox Fruits", 2753915549),
                g("Brookhaven RP", 4924922222),
                g("Arsenal", 286090429),
            ],
            selected: 0,
            close_on_launch: false,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lunar-bx")
        .join("config.json")
}

fn load_config() -> Config {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
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

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Play,
    Games,
    Settings,
}

struct App {
    cfg: Config,
    tab: Tab,
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
        v.widgets.inactive.rounding = 8.0.into();
        v.widgets.hovered.rounding = 8.0.into();
        v.widgets.active.rounding = 8.0.into();
        v.extreme_bg_color = CARD;
        cc.egui_ctx.set_visuals(v);

        let mut cfg = load_config();
        if cfg.selected >= cfg.games.len() {
            cfg.selected = 0;
        }
        Self {
            cfg,
            tab: Tab::Play,
            new_name: String::new(),
            new_id: String::new(),
            status: String::new(),
        }
    }

    /// Opens the installed Roblox client through its official deep link.
    /// Roblox uses whichever account is signed in on this machine.
    fn launch(&mut self, ctx: &egui::Context) {
        let Some(game) = self.cfg.games.get(self.cfg.selected).cloned() else {
            self.status = "Add a game first.".into();
            return;
        };
        let url = format!("roblox://experiences/start?placeId={}", game.place_id);
        match open::that(&url) {
            Ok(_) => {
                self.status = format!("Launching {}…", game.name);
                if self.cfg.close_on_launch {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Err(_) => {
                self.status = "Couldn't open Roblox. Check that it's installed and you're signed in.".into();
            }
        }
    }

    fn nav_button(&mut self, ui: &mut egui::Ui, label: &str, tab: Tab) {
        let active = self.tab == tab;
        let btn = egui::Button::new(
            RichText::new(label)
                .size(15.0)
                .color(if active { TEXT } else { MUTED }),
        )
        .fill(if active { CARD_ACTIVE } else { Color32::TRANSPARENT })
        .stroke(Stroke::NONE);
        if ui.add_sized([158.0, 40.0], btn).clicked() {
            self.tab = tab;
        }
    }

    fn play_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(24.0);
        match self.cfg.games.get(self.cfg.selected).cloned() {
            Some(game) => {
                ui.label(RichText::new(&game.name).size(40.0).strong());
                ui.label(RichText::new(format!("Place ID {}", game.place_id)).color(MUTED));
                ui.add_space(22.0);
                let launch = egui::Button::new(
                    RichText::new("LAUNCH").size(22.0).strong().color(BG),
                )
                .fill(ACCENT);
                if ui.add_sized([280.0, 64.0], launch).clicked() {
                    self.launch(ctx);
                }
                ui.add_space(8.0);
                ui.label(RichText::new(&self.status).color(MUTED));
            }
            None => {
                ui.label(RichText::new("No games yet").size(32.0).strong());
                ui.label(RichText::new("Open the Games tab and add a place ID.").color(MUTED));
            }
        }

        ui.add_space(36.0);
        ui.label(RichText::new("Your games").size(18.0).strong());
        ui.add_space(8.0);
        let mut pick = None;
        ui.horizontal_wrapped(|ui| {
            for (i, g) in self.cfg.games.iter().enumerate() {
                let active = i == self.cfg.selected;
                let btn = egui::Button::new(RichText::new(&g.name).size(15.0))
                    .fill(if active { CARD_ACTIVE } else { CARD })
                    .stroke(Stroke::new(1.5, if active { ACCENT } else { Color32::TRANSPARENT }));
                if ui.add_sized([160.0, 72.0], btn).clicked() {
                    pick = Some(i);
                }
            }
        });
        if let Some(i) = pick {
            self.cfg.selected = i;
            save_config(&self.cfg);
        }
    }

    fn games_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(RichText::new("Games").size(32.0).strong());
        ui.add_space(16.0);

        let mut remove = None;
        for (i, g) in self.cfg.games.iter().enumerate() {
            egui::Frame::none()
                .fill(CARD)
                .rounding(10.0)
                .inner_margin(Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&g.name).size(16.0).strong());
                            ui.label(RichText::new(g.place_id.to_string()).color(MUTED));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("Remove").color(DANGER)).clicked() {
                                remove = Some(i);
                            }
                        });
                    });
                });
            ui.add_space(6.0);
        }
        if let Some(i) = remove {
            self.cfg.games.remove(i);
            if self.cfg.selected >= self.cfg.games.len() {
                self.cfg.selected = 0;
            }
            save_config(&self.cfg);
        }

        ui.add_space(18.0);
        ui.label(RichText::new("Add a game").size(18.0).strong());
        ui.label(
            RichText::new("The place ID is the number in the game's Roblox URL.").color(MUTED),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [220.0, 32.0],
                egui::TextEdit::singleline(&mut self.new_name).hint_text("Game name"),
            );
            ui.add_sized(
                [160.0, 32.0],
                egui::TextEdit::singleline(&mut self.new_id).hint_text("Place ID"),
            );
            let add = egui::Button::new(RichText::new("Add game").color(BG)).fill(ACCENT);
            if ui.add_sized(Vec2::new(100.0, 32.0), add).clicked() {
                if let Ok(id) = self.new_id.trim().parse::<u64>() {
                    let name = self.new_name.trim();
                    self.cfg.games.push(Game {
                        name: if name.is_empty() { format!("Game {id}") } else { name.into() },
                        place_id: id,
                    });
                    self.new_name.clear();
                    self.new_id.clear();
                    save_config(&self.cfg);
                }
            }
        });
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(RichText::new("Settings").size(32.0).strong());
        ui.add_space(16.0);
        if ui
            .checkbox(&mut self.cfg.close_on_launch, "Close the launcher after launching")
            .changed()
        {
            save_config(&self.cfg);
        }
        ui.add_space(16.0);
        ui.label(RichText::new("Config file").strong());
        ui.label(RichText::new(config_path().display().to_string()).color(MUTED));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("nav")
            .exact_width(190.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(PANEL).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Lunar BX").size(24.0).strong().color(ACCENT));
                ui.add_space(24.0);
                self.nav_button(ui, "Play", Tab::Play);
                self.nav_button(ui, "Games", Tab::Games);
                self.nav_button(ui, "Settings", Tab::Settings);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new("v0.1.0").color(MUTED).size(12.0));
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(Margin::same(28.0)))
            .show(ctx, |ui| match self.tab {
                Tab::Play => self.play_tab(ui, ctx),
                Tab::Games => self.games_tab(ui),
                Tab::Settings => self.settings_tab(ui),
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([940.0, 580.0])
            .with_min_inner_size([760.0, 480.0])
            .with_title("Lunar BX"),
        ..Default::default()
    };
    eframe::run_native("Lunar BX", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
