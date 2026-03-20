pub mod accordion;
mod overview;
pub use overview::show_overview;
mod segments;
pub use segments::{
    CustomerSegment, CustomerSegmentPage, Gain, Job, Pain, customer_sidepanel, show_customer,
};

pub mod product;
pub use product::{ValuePropPage, product_sidepanel, show_product};
