pub mod router;
pub mod openai;
pub mod admin;

pub use router::{web_route_wrapper, web_routes};
pub use openai::{chat, models};
pub use admin::{request, api_key, config, provider};