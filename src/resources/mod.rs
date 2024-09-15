use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

mod class;
mod group;
mod namespace;
mod object;
mod permission;
mod user;

pub use self::class::{Class, ClassGet, ClassPatch, ClassPost};
pub use self::group::{Group, GroupGet, GroupPatch, GroupPost};
pub use self::namespace::{Namespace, NamespaceGet, NamespacePatch, NamespacePost};
pub use self::user::{User, UserGet, UserPatch, UserPost};
pub use crate::types::{FilterOperator, QueryFilter};

use crate::endpoints::Endpoint;

// ApiResource trait
pub trait ApiResource: Default {
    type GetParams: Serialize + Debug + Default;
    type GetOutput: DeserializeOwned + Debug;
    type PostParams: Serialize + Debug;
    type PostOutput: DeserializeOwned + Debug;
    type PatchParams: Serialize + Debug;
    type PatchOutput: DeserializeOwned + Debug;
    type DeleteParams: Serialize + Debug;
    type DeleteOutput: DeserializeOwned + Debug;

    fn endpoint(&self) -> Endpoint;
    fn build_params(filters: Vec<(String, FilterOperator, String)>) -> Vec<QueryFilter>;
}
