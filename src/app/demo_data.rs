use std::str::FromStr;

use crate::App;
use crate::app::ProductPage;
use crate::app::Tab;
use crate::app::pages::product::products_window::Product;
use uuid::Uuid;

pub fn load_demo_data(cc: &eframe::CreationContext<'_>) -> App {
    let mut demo_app: App;
    if let Some(storage) = cc.storage {
        demo_app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    } else {
        demo_app = App {
            label: String::new(),
            value: 0.0,
            tab: Tab::Product,
            product_page: ProductPage::default(),
        };
    }

    let product_1 = Product {
        name: "Product 1".to_string(),
        description: "My product description".to_string(),
        id: Uuid::from_str("e3142c46-5ac5-4425-8080-a8faff6e3ae4").unwrap(),
    };
    let product_2 = Product {
        name: "Product 2".to_string(),
        description: "My product description".to_string(),
        id: Uuid::from_str("93a7b2b5-ce26-4078-bce6-ca7d2d941b70").unwrap(),
    };

    let prod_vec = &mut demo_app.product_page.products_state.products;
    // To be able to keep a bit of state chache from any products adding during demo,
    // While ensuring the first two products are always defined here.
    if prod_vec.len() >= 1 {
        prod_vec[0] = product_1;
    } else {
        prod_vec.push(product_1);
    }

    if prod_vec.len() >= 2 {
        prod_vec[1] = product_2;
    } else {
        prod_vec.push(product_2);
    }
    demo_app
}
