use api::GroupVersion;
pub use policy::*;
pub use policy_attachment::*;
pub use role_attachment::*;

mod policy;
mod policy_attachment;
mod role_attachment;

lazy_static! {
    pub static ref API_VERSION: GroupVersion = GroupVersion {
        group: "rbac".to_string(),
        version: "v1alpha1".to_string(),
    };
}
