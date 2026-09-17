pub mod router;
pub mod openai;
pub mod admin;

pub use router::WebRouter;
pub use openai::{chat, models};
pub use admin::{request, api_key, config, provider};