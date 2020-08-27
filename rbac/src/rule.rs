use std::fmt::Display;

use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    sub: Guid,
    obj: Guid,
    act: String,
}

impl Rule {
    pub fn build_id(sub: &str, obj: &str, act: &str) -> Guid {
        Self::build_guid(&format!("{}-{}-{}", sub, obj, act))
    }

    pub(crate) fn new(sub: Guid, obj: Guid, act: String) -> Self {
        let id = Self::build_id(&sub.to_string(), &obj.to_string(), &act);
        Rule {
            id,
            rev: None,
            sub,
            obj,
            act,
        }
    }

    pub fn subject(&self) -> &Guid {
        &self.sub
    }

    pub fn object(&self) -> &Guid {
        &self.obj
    }

    pub fn action(&self) -> &str {
        &self.act
    }
}

impl Entity for Rule {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("rule:{}", id))
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    subject: Guid,
    role: String,
}

impl RoleAssignment {
    pub fn build_id<D: Display, R: Display>(sub: D, role: R) -> Guid {
        Self::build_guid(&format!("{}-{}", role, sub))
    }

    pub(crate) fn new(subject: Guid, role: &str) -> Self {
        let id = Self::build_id(&subject, &role);
        RoleAssignment {
            id,
            rev: None,
            subject,
            role: role.to_string(),
        }
    }

    pub fn subject(&self) -> &Guid {
        &self.subject
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}

impl Entity for RoleAssignment {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("role_assignment:{}", id))
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}
