use time::OffsetDateTime;

pub trait Event {
    type AggregateId;

    fn aggregate_id(&self) -> Self::AggregateId;
    fn timestamp(&self) -> OffsetDateTime;
}
