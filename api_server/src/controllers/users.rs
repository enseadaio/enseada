use api::Client;
use api::tonic::Status;
use api::tonic::transport::Channel;
use api::users::v1alpha1::users_client::UsersClient;
use api::users::v1alpha1::WatchUsersRequest;

use crate::error::Error;
use slog::Logger;
use crossbeam_channel::Receiver;

pub async fn start(logger: Logger, client: &Client, ready_channel: Receiver<()>) -> Result<(), Error> {
    slog::debug!(logger, "Waiting for gRPC server to start");
    // return Ok(());
    ready_channel.recv().unwrap();
    slog::info!(logger, "Starting users controller");
    loop {
        let uc = client.users();
        if let Err(err) = loop_stream(logger.clone(), uc).await {
            slog::error!(logger, "{}", err);
        }
    }

    Ok(())
}

async fn loop_stream(logger: Logger, mut client: UsersClient<Channel>) -> Result<(), Status> {
    let mut stream = client.watch_users(WatchUsersRequest {}).await?.into_inner();
    while let Some(s) = stream.message().await? {
        slog::warn!(logger, "User Event: {:?}", s);
    }

    Ok(())
}
