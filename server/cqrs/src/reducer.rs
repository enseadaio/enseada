pub trait Reducer<Aggregate, Event: crate::event::Event> {
    fn apply(&self, aggregate: Aggregate, event: Event) -> Option<Aggregate>;
}

impl<A, E, F> Reducer<A, E> for F
where
    E: crate::event::Event,
    F: Fn(A, E) -> Option<A>,
{
    fn apply(&self, aggregate: A, event: E) -> Option<A> {
        self(aggregate, event)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::Event;
    use time::OffsetDateTime;

    fn reduce<A, E: Event, I: IntoIterator<Item = E>, R: Reducer<A, E>>(
        agg: A,
        events: I,
        reducer: R,
    ) -> Option<A> {
        events
            .into_iter()
            .try_fold(agg, |agg, ev| reducer.apply(agg, ev))
    }

    enum TestEvent {
        Add(u8),
        Sub(u8),
    }

    impl Event for TestEvent {
        type AggregateId = u8;

        fn aggregate_id(&self) -> Self::AggregateId {
            0
        }

        fn timestamp(&self) -> OffsetDateTime {
            OffsetDateTime::now_utc()
        }
    }

    struct TestReducer;

    impl Reducer<u8, TestEvent> for TestReducer {
        fn apply(&self, aggregate: u8, event: TestEvent) -> Option<u8> {
            match event {
                TestEvent::Add(n) => Some(aggregate + n),
                TestEvent::Sub(n) => Some(aggregate - n),
            }
        }
    }

    impl Event for u8 {
        type AggregateId = u8;

        fn aggregate_id(&self) -> Self::AggregateId {
            0
        }

        fn timestamp(&self) -> OffsetDateTime {
            OffsetDateTime::now_utc()
        }
    }

    #[test]
    fn it_works_with_closures() {
        let sum = reduce(0, [1, 2, 3], |agg, ev| Some(agg + ev));
        assert_eq!(sum, Some(6));
    }

    #[test]
    fn it_works_with_structs() {
        let sum = reduce(0, [TestEvent::Add(6), TestEvent::Sub(3)], TestReducer);
        assert_eq!(sum, Some(3));
    }
}
