use std::str::FromStr as _;

use crate::App;
use crate::app::ProductPage;
use crate::app::Tab;
use crate::app::pages::product::Feature;
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
        name: "Product 1".to_owned(),
        description: "My product description".to_owned(),
        id: Uuid::from_str("e3142c46-5ac5-4425-8080-a8faff6e3ae4")
            .expect("hardcoded UUID is valid"),
        notes: String::new(),
        expanded: false,
    };
    let product_2 = Product {
        name: "Product 2".to_owned(),
        description: "My product description".to_owned(),
        id: Uuid::from_str("93a7b2b5-ce26-4078-bce6-ca7d2d941b70")
            .expect("hardcoded UUID is valid"),
        notes: String::new(),
        expanded: false,
    };

    let prod_vec = &mut demo_app.product_page.products_state.products;
    // To be able to keep a bit of state cache from any products added during demo,
    // while ensuring the first two products are always defined here.
    let demo_products = [product_1, product_2];
    for (i, product) in demo_products.into_iter().enumerate() {
        if let Some(slot) = prod_vec.get_mut(i) {
            *slot = product;
        } else {
            prod_vec.push(product);
        }
    }

    let feature_1 = Feature {
        name: "Feature 1".to_owned(),
        description: "My feature description".to_owned(),
        id: Uuid::from_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890")
            .expect("hardcoded UUID is valid"),
        status: "Draft".to_owned(),
        notes: String::new(),
        user_story: String::new(),
        acceptance_criteria: String::new(),
        expanded: false,
    };
    let feature_2 = Feature {
        name: "Feature 2".to_owned(),
        description: "My feature description".to_owned(),
        id: Uuid::from_str("b2c3d4e5-f6a7-8901-bcde-f12345678901")
            .expect("hardcoded UUID is valid"),
        status: "Draft".to_owned(),
        notes: String::new(),
        user_story: String::new(),
        acceptance_criteria: String::new(),
        expanded: false,
    };

    let feat_vec = &mut demo_app.product_page.features_state.features;
    // Same pattern as products: keep any user-added features beyond the demo slots,
    // while ensuring the first two features are always defined here.
    let demo_features = [feature_1, feature_2];
    for (i, feature) in demo_features.into_iter().enumerate() {
        if let Some(slot) = feat_vec.get_mut(i) {
            *slot = feature;
        } else {
            feat_vec.push(feature);
        }
    }

    demo_app
}
