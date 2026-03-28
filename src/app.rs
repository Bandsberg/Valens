mod pages;
use pages::{
    CustomerSegmentPage, ValuePropPage, customer_sidepanel, product_sidepanel, show_customer,
    show_overview, show_product,
};
mod demo_data;
use demo_data::load_demo_data;
#[cfg(not(target_arch = "wasm32"))]
mod db;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum Mode {
    Demo,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Tab {
    ValueProp,
    Customer,
    Overview,
}

/// We derive Deserialize/Serialize so we can persist UI state (tab, window toggles) on shutdown.
/// Entity data (products, features, etc.) is persisted in `SQLite` — those fields are serde-skipped.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    tab: Tab,
    valueprop_page: ValuePropPage,
    customer_segment_page: CustomerSegmentPage,
    /// Open handle to the `SQLite` database (native only, not serialized).
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    db: Option<db::Database>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::ValueProp,
            valueprop_page: ValuePropPage::default(),
            customer_segment_page: CustomerSegmentPage::default(),
            #[cfg(not(target_arch = "wasm32"))]
            db: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, mode: Mode) -> Self {
        match mode {
            Mode::Demo => load_demo_data(cc),
            Mode::Production => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let db_path = eframe::storage_dir("valens")
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("valens.db");
                    let database = db::Database::open(&db_path).expect("Failed to open database");
                    // Load UI state (tab, window toggles) from eframe::Storage.
                    let mut app: Self = cc
                        .storage
                        .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
                        .unwrap_or_default();
                    // Populate entity data from SQLite.
                    database
                        .load_into(&mut app)
                        .expect("Failed to load data from database");
                    app.db = Some(database);
                    app
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // Production mode is native-only; this branch is unreachable in practice.
                    Self::default()
                }
            }
        }
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Persist UI state (tab, window toggles).
        eframe::set_value(storage, eframe::APP_KEY, self);
        // Persist entity data to SQLite.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mut database) = self.db.take() {
            if let Err(e) = database.save(&self.valueprop_page, &self.customer_segment_page) {
                log::error!("Failed to save data to database: {e}");
            }
            self.db = Some(database);
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
                ui.separator();
                ui.selectable_value(&mut self.tab, Tab::ValueProp, "Value Proposition");
                ui.selectable_value(&mut self.tab, Tab::Customer, "Customer Segments");
                ui.selectable_value(&mut self.tab, Tab::Overview, "Overview");
            });
        });

        // Only the two data-entry tabs have a side panel (Overview is read-only).
        match self.tab {
            Tab::ValueProp => product_sidepanel(self, ui),
            Tab::Customer => customer_sidepanel(self, ui),
            Tab::Overview => {} // no side panel
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let ctx = ui.ctx().clone();
            match self.tab {
                Tab::ValueProp => show_product(self, &ctx, ui),
                Tab::Customer => show_customer(self, &ctx, ui),
                Tab::Overview => show_overview(self, &ctx, ui),
            }
        });
    }
}
