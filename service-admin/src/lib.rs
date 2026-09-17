//! 管理端业务实现：repository 只做单表原子操作，service 编排单表逻辑，
//! manager 负责跨表组合。

pub mod manager;
pub mod repository;
pub mod service;
