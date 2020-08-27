use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};

use api::users::v1beta1::UserModel;
use users::{User, UserService};

mod rbac;
mod user;

pub fn mount(cfg: &mut ServiceConfig) {
    let couch = &crate::couchdb::SINGLETON;
    let db = couch.database(crate::couchdb::name::USERS, true);
    let service = UserService::new(db);
    cfg.data(service);

    // CRUD
    cfg.service(self::user::me);
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
