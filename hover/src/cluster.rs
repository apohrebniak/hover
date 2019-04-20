use crate::common::Address;

/**Member of the cluster*/
#[derive(PartialEq, Eq, Hash)]
pub struct Member {
    pub id: String,
    pub addr: Address,
    pub active: bool,
}
