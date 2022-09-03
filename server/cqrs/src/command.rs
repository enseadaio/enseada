use async_trait::async_trait;

#[derive(Debug)]
pub struct ValidationError {
    msg: String,
}

impl ValidationError {
    pub fn message(&self) -> &str {
        &self.msg
    }
}

impl<T: ToString> From<T> for ValidationError {
    fn from(s: T) -> Self {
        Self { msg: s.to_string() }
    }
}

#[derive(Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn errors(&self) -> &[ValidationError] {
        self.errors.as_slice()
    }
}

impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }
}

#[async_trait]
pub trait CommandHandler<Command, Event: crate::event::Event> {
    async fn validate(
        &self,
        command: Command,
    ) -> Result<(Event::AggregateId, Command), ValidationErrors>;

    fn apply(&self, id: Event::AggregateId, command: Command) -> Vec<Event>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::Event;
    use time::OffsetDateTime;

    #[derive(Debug)]
    pub struct TestCommand(bool);

    pub struct TestEvent;

    impl Event for TestEvent {
        type AggregateId = u8;

        fn aggregate_id(&self) -> Self::AggregateId {
            1
        }

        fn timestamp(&self) -> OffsetDateTime {
            OffsetDateTime::now_utc()
        }
    }

    pub struct TestHandler;

    #[async_trait]
    impl CommandHandler<TestCommand, TestEvent> for TestHandler {
        async fn validate(
            &self,
            command: TestCommand,
        ) -> Result<(u8, TestCommand), ValidationErrors> {
            if command.0 {
                Ok((1, command))
            } else {
                Err(vec!["ValidationError".into()].into())
            }
        }

        fn apply(&self, _id: u8, _command: TestCommand) -> Vec<TestEvent> {
            vec![TestEvent]
        }
    }

    #[tokio::test]
    async fn it_works() {
        let handler = TestHandler;

        let (id, cmd) = handler.validate(TestCommand(true)).await.unwrap();
        let events = handler.apply(id, cmd);

        assert_eq!(events.len(), 1);

        let errors = handler.validate(TestCommand(false)).await.unwrap_err();
        assert_eq!(errors.errors.len(), 1);
    }
}
