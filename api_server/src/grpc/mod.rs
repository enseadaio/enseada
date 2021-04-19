use futures::FutureExt;
use slog::Logger;

use api::tonic::transport::Server;
use couchdb::Couch;

use crate::config::Configuration;
use crate::resources::ResourceManager;
use crate::ServerResult;

mod users;

pub async fn start(cfg: Configuration, logger: Logger, couch: &Couch) -> ServerResult {
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
        .await?;

    Ok(())
}
