use crate::common::Address;

/**Member of the cluster*/
#[derive(PartialEq, Eq, Hash)]
pub struct Member {
    id: String,
    addr: Address,
    active: bool,
}
