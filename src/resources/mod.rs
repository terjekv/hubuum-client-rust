use serde::{de::DeserializeOwned, Serialize};

mod class;
mod group;
mod namespace;
mod object;
mod permission;
mod user;

pub use self::class::{Class, ClassGet, ClassPatch, ClassPost};
pub use self::user::{User, UserGet, UserPatch, UserPost};

use crate::endpoints::Endpoint;
use crate::types::FilterOperator;

// ApiResource trait
pub trait ApiResource: Default {
    type GetParams: Serialize + std::fmt::Debug + Default;
    type GetOutput: DeserializeOwned;
    type PostParams: Serialize + std::fmt::Debug;
    type PostOutput: DeserializeOwned;
    type PatchParams: Serialize + std::fmt::Debug;
    type PatchOutput: DeserializeOwned;
    type DeleteParams: Serialize + std::fmt::Debug;
    type DeleteOutput: DeserializeOwned;

    fn endpoint(&self) -> Endpoint;
    fn build_params(filters: Vec<(String, FilterOperator, String)>) -> Self::GetParams;
}
