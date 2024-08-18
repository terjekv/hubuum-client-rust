use serde::{Deserialize, Serialize};

use crate::endpoints::Endpoint;
use crate::resources::ApiResource;
use crate::types::FilterOperator;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserGet {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserPost {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserPatch {
    pub username: Option<String>,
    pub email: Option<String>,
}

impl ApiResource for User {
    type GetParams = UserGet;
    type GetOutput = User;
    type PostParams = UserPost;
    type PostOutput = User;
    type PatchParams = UserPatch;
    type PatchOutput = User;
    type DeleteParams = ();
    type DeleteOutput = ();

    fn endpoint(&self) -> Endpoint {
        Endpoint::GetUser
    }

    fn build_params(filters: Vec<(String, FilterOperator, String)>) -> Self::GetParams {
        let mut params = UserGet::default();

        for (field, op, value) in filters {
            match (field.as_str(), op) {
                ("username", FilterOperator::Eq) => params.username = Some(value),
                ("email", FilterOperator::Eq) => params.email = Some(value),
                // Add more cases as needed
                _ => {} // Ignore unknown fields or operators
            }
        }
        params
    }
}
