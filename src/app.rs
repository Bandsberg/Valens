mod pages;
use pages::{
    CustomerPage, ProductPage, customer_sidepanel, product_sidepanel, show_customer, show_product,
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
    Product,
    Customer,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    tab: Tab,
    product_page: ProductPage,
    customer_page: CustomerPage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            tab: Tab::Product,
            product_page: ProductPage::default(),
            customer_page: CustomerPage::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, mode: Mode) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        //

        match mode {
            // Use demo data if the mode is demo
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
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

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
                ui.selectable_value(&mut self.tab, Tab::Product, "Products & Services");
                ui.selectable_value(&mut self.tab, Tab::Customer, "Customer segment");
            });
        });

        if self.tab == Tab::Product {
            product_sidepanel(self, ctx);
        }
        if self.tab == Tab::Customer {
            customer_sidepanel(self, ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            //
            match self.tab {
                Tab::Product => {
                    show_product(self, ctx, ui);
                }
                Tab::Customer => {
                    show_customer(self, ctx, ui);
                }
            }
        });
    }
}
