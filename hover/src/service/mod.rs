pub mod cluster_service;
pub mod connection_service;
pub mod discovery_service;

/**Common trait for all runnable services*/
pub trait Service {
    fn start(&self);
}
