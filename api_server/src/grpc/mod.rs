use api::tonic::transport::Server;
use couchdb::Couch;

use crate::resources::ResourceManager;
use crate::ServerResult;
use slog::Logger;
use crate::config::Configuration;
use crossbeam_channel::{Sender, Receiver};
use futures::FutureExt;

mod users;

pub async fn start<'a>(cfg: &'a Configuration, logger: Logger, couch: &'a Couch) -> ServerResult {
    slog::debug!(logger, "Starting gRPC server");
    let addr = cfg.grpc().address();

    let users_db = couch.database("users", true);
    let users_mgr = ResourceManager::new(users_db, "user");
    users_mgr.init().await?;
    let users = users::UsersService::new(logger.new(slog::o!("service" => "users")), users_mgr);

    slog::info!(logger, "GRPC server listening on {}", addr);
    Server::builder()
        .add_service(users)
        .serve(addr)
        // .map(|res| {
        //     ready_channel.send(()).unwrap();
        //     res
        // })
        .await?;

    Ok(())
}

pub fn ready_channel() -> (Sender<()>, Receiver<()>) {
    crossbeam_channel::unbounded()
}
