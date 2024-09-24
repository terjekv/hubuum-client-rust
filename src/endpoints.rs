use crate::types::BaseUrl;

pub enum Endpoint {
    Login,
    LoginWithToken,
    Users,
    Groups,
    Classes,
    Namespaces,
    // ... other endpoints
    ObjectsInClass,

    ClassRelations,
    ClassRelationsTransitive,
    ClassRelationsTransitiveToClass,

    ObjectRelation,
    ObjectRelations,
}

impl Endpoint {
    pub fn path(&self) -> &'static str {
        match self {
            Endpoint::Login => "/api/v0/auth/login",
            Endpoint::LoginWithToken => "/api/v0/auth/validate",
            Endpoint::Users => "/api/v1/iam/users/",
            Endpoint::Groups => "/api/v1/iam/groups/",
            Endpoint::Classes => "/api/v1/classes/",
            Endpoint::Namespaces => "/api/v1/namespaces/",

            Endpoint::ObjectsInClass => "/api/v1/classes/{class_id}/",

            Endpoint::ClassRelations => "/api/v1/classes/{class_id}/relations/",
            Endpoint::ClassRelationsTransitive => {
                "/api/v1/classes/{class_id}/relations/transitive/"
            }
            Endpoint::ClassRelationsTransitiveToClass => {
                "/api/v1/classes/{class_id}/relations/transitive/class/{to_class_id}/"
            }

            Endpoint::ObjectRelations => "/api/v1/classes/{class_id}/{from_object_id}/relations/",

            Endpoint::ObjectRelation => {
                "/api/v1/classes/{class_id}/{from_object_id}/relations/{to_class_id}/{to_object_id}"
            }
        }
    }

    pub fn complete(&self, baseurl: &BaseUrl) -> String {
        format!(
            "{}{}",
            baseurl.with_trailing_slash(),
            self.trim_start_matches('/')
        )
    }

    pub fn trim_start_matches(&self, prefix: char) -> &str {
        self.path().trim_start_matches(prefix)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use yare::parameterized;

    #[parameterized(
        login = { Endpoint::Login, "/api/v0/auth/login" },
        get_user = { Endpoint::Users, "/api/v1/iam/users/" },
        get_class = { Endpoint::Classes, "/api/v1/classes/" }
    )]
    fn test_endpoint_path(endpoint: Endpoint, expected: &str) {
        assert_eq!(endpoint.path(), expected);
    }

    #[parameterized(
        login = { Endpoint::Login, '/', "api/v0/auth/login" },
        get_user = { Endpoint::Users, '/', "api/v1/iam/users/" },
        get_class = { Endpoint::Classes, '/', "api/v1/classes/" }
    )]
    fn test_trim_start_matches(endpoint: Endpoint, prefix: char, expected: &str) {
        assert_eq!(endpoint.trim_start_matches(prefix), expected);
    }

    #[parameterized(
        api_login = { Endpoint::Login, BaseUrl::from_str("https://api.example.com").unwrap(), "https://api.example.com/api/v0/auth/login" },
        api_get_user = { Endpoint::Users, BaseUrl::from_str("https://api.example.com").unwrap(), "https://api.example.com/api/v1/iam/users/" },
        foo_login_with_token = { Endpoint::LoginWithToken, BaseUrl::from_str("https://foo.bar.com").unwrap(), "https://foo.bar.com/api/v0/auth/validate" },
    )]
    fn test_complete(endpoint: Endpoint, baseurl: BaseUrl, expected: &str) {
        assert_eq!(endpoint.complete(&baseurl), expected);
    }
}
