mod pages;
use pages::{
    CustomerSegmentPage, ValuePropPage, customer_sidepanel, product_sidepanel, show_customer,
    show_overview, show_product,
};
mod demo_data;
use demo_data::load_demo_data;

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

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    tab: Tab,
    valueprop_page: ValuePropPage,
    customer_segment_page: CustomerSegmentPage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::ValueProp,
            valueprop_page: ValuePropPage::default(),
            customer_segment_page: CustomerSegmentPage::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, mode: Mode) -> Self {
        match mode {
            Mode::Demo => load_demo_data(cc),
            Mode::Production => {
                // Load previous app state (if any).
                // Note that you must enable the `persistence` feature for this to work.
                cc.storage
                    .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
                    .unwrap_or_default()
            }
        }
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

        if self.tab == Tab::ValueProp {
            product_sidepanel(self, ctx);
        }
        if self.tab == Tab::Customer {
            customer_sidepanel(self, ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::ValueProp => {
                show_product(self, ctx, ui);
            }
            Tab::Customer => {
                show_customer(self, ctx, ui);
            }
            Tab::Overview => {
                show_overview(self, ctx, ui);
            }
        });
    }
}
