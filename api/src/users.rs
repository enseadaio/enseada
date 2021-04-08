pub mod v1alpha1 {
    use crate::watch::{FromResource, Event};
    use crate::watch::v1alpha1::EventType;
    include!(concat!(env!("OUT_DIR"), concat!("/enseada.users.v1alpha1.rs")));

    impl FromResource<User> for UserEvent {
        fn from_res(event_type: EventType, res: User) -> Self {
            Self {
                r#type: event_type as i32,
                user: Some(res),
            }
        }
    }

    impl Event<User> for UserEvent {
        fn event_type(&self) -> EventType {
            EventType::from_i32(self.r#type).unwrap()
        }

        fn into_inner(self) -> Option<User> {
            self.user
        }
    }
}
