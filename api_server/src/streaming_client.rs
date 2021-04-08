use api::Client;
use api::meta::v1alpha1::Metadata;
use api::tonic::transport::Channel;
use api::users::v1alpha1::{User, WatchUsersRequest};
use api::users::v1alpha1::users_client::UsersClient;

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("http://[::1]:9624", None)?;
    let mut uc = client.users();
    loop {
        listen(&mut uc).await?;
        eprintln!("Reconnecting");
    }

    Ok(())
}

async fn listen(uc: &mut UsersClient<Channel>) -> Result<(), Error> {
    let request = api::tonic::Request::new(WatchUsersRequest {});
    let mut stream = uc.watch_users(request).await?.into_inner();
    while let res = stream.message().await {
        match res {
            Ok(Some(user)) => eprintln!("USER={:?}", user),
            Ok(None) => {},
            Err(err) => {
                eprintln!("{}", err);
                return Ok(())
            },
        };
    }

    Ok(())
}

