use crate::watch::v1alpha1::EventType;

pub mod v1alpha1 {
    include!(concat!(env!("OUT_DIR"), concat!("/enseada.watch.v1alpha1.rs")));
}

pub trait FromResource<T> {
    fn from_res(event_type: EventType, res: T) -> Self;
}

pub trait Event<T> {
    fn event_type(&self) -> EventType;
    fn into_inner(self) -> Option<T>;
}
