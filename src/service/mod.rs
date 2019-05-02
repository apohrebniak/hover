pub mod broadcast;
pub mod connection;
pub mod membership;

/**Common trait for all runnable services*/
pub trait Service {
    fn start(&self);
}
