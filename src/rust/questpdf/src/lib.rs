//! A Rust port of QuestPDF's layout and fluent composition model.
//! Native Skia is shared with the .NET library; layout and applications run in Rust.
pub const SOURCE_VERSION: &str = "2026.9.1";
pub const SKIA_ABI_VERSION: i32 = 26;
pub const QPDF_ABI_VERSION: i32 = 4;
pub mod document_operation;
pub mod dynamic;
pub use dynamic::{
    DynamicComponentComposeResult, DynamicContext, DynamicElement, IDynamicComponent,
};
mod canvas;
pub mod semantic;
pub use semantic::{SemanticAttribute, SemanticAttributeValue, SemanticInfo};
mod drawing;
pub mod qpdf;
pub use document_operation::*;
pub mod elements;
pub mod fluent;
pub mod font_manager;
pub use font_manager::FontManager;
pub mod helpers;
pub mod infrastructure;
pub use helpers::{FontFeatures, colors};
pub mod companion;
mod native;
pub mod runtime;
pub mod settings;
pub use settings::{GlobalSettings, Settings};
mod xmp;
pub use drawing::Renderer;
pub use elements::Element;
pub use fluent::*;
pub use infrastructure::*;
