use std::sync::{Arc, RwLock};

use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};

use api::users::v1beta1::UserModel;
use enseada::couchdb::db::Database;
use enseada::events::EventBus;
use users::{User, UserService};

use crate::config::Configuration;

mod rbac;
mod user;

pub fn mount(db: Database, bus: Arc<RwLock<EventBus>>) -> Box<impl FnOnce(&mut ServiceConfig)> {
    Box::new(|cfg: &mut ServiceConfig| {
        let service = UserService::new(db, bus);
        cfg.data(service);

        // Profile
        cfg.service(self::user::me);
        cfg.service(self::rbac::list_capabilities);

        // Users
        cfg.service(self::user::list);
        cfg.service(self::user::register);
        cfg.service(self::user::get);
        cfg.service(self::user::update);
        cfg.service(self::user::delete);

        // RBAC
        cfg.service(self::rbac::list_permissions);
        cfg.service(self::rbac::add_permission);
        cfg.service(self::rbac::remove_permission);
        cfg.service(self::rbac::list_roles);
        cfg.service(self::rbac::add_role);
        cfg.service(self::rbac::remove_role);
    })
}

#[derive(Debug, Deserialize)]
pub struct UsernamePathParam {
    pub username: String,
}

fn map_user(user: &User) -> UserModel {
    UserModel {
        username: user.username().to_string(),
        enabled: user.is_enabled(),
    }
}

#[inline]
fn map_owned_user(user: User) -> UserModel {
    map_user(&user)
}
