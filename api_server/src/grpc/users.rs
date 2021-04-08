use actix::Arbiter;
use futures::channel::mpsc;

use api::tonic::{Request, Response, Status};
use api::users::v1alpha1::{CreateUserRequest, DeleteUserRequest, GetUserRequest, User, WatchUsersRequest, UserEvent};
use api::users::v1alpha1::users_server::{Users, UsersServer};

use crate::resources::{ResourceManager, Watcher};
use slog::Logger;

pub struct UsersService {
    logger: Logger,
    manager: ResourceManager<User>,
    arbiter: Arbiter,
}

impl UsersService {
    pub fn new(logger: Logger, manager: ResourceManager<User>) -> UsersServer<UsersService> {
        UsersServer::new(UsersService {
            logger,
            manager,
            arbiter: Arbiter::new(),
        })
    }
}

#[api::tonic::async_trait]
impl Users for UsersService {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<User>, Status> {
        let name = request.into_inner().name;
        slog::info!(self.logger, "Got request for user {}!", &name);

        let user = self.manager.get(&name).await?;
        slog::info!(self.logger, "Got user {:?}", &user);
        Ok(Response::new(user))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let user = request.into_inner().user.unwrap();
        let name = user.metadata.as_ref().unwrap().name.to_string();
        slog::info!(self.logger, "Creating user {:?}!", &user);

        let user = self.manager.put(&name, user).await?;
        Ok(Response::new(user))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().name;
        slog::info!(self.logger, "Got deletion request for user {}!", &name);

        self.manager.delete(&name).await?;
        Ok(Response::new(()))
    }

    type WatchUsersStream = mpsc::Receiver<Result<UserEvent, Status>>;
    async fn watch_users(
        &self,
        _request: Request<WatchUsersRequest>,
    ) -> Result<Response<Self::WatchUsersStream>, Status> {
        slog::info!(self.logger, "Got request to watch all users");

        let rx = Watcher::start(self.logger.new(slog::o!("watcher" => true)), self.manager.clone(), &self.arbiter.handle());
        Ok(Response::new(rx))
    }
    //
    // type WatchUserStream = mpsc::Receiver<Result<User, Status>>;
    // async fn watch_user(
    //     &self,
    //     request: Request<WatchUserRequest>,
    // ) -> Result<Response<Self::WatchUserStream>, Status> {
    //     slog::info!(self.logger, "Got request to watch user {}", request.into_inner().name);
    //     let (tx, rx) = mpsc::channel();
    //
    //     let w = Watcher::new(self.db.clone(), tx);
    //     Supervisor::start_in_arbiter(&self.arbiter.handle(), move |_| w);
    //
    //     Ok(Response::new(rx.map(Ok)))
    // }
}
