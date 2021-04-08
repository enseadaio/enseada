use api::users::v1alpha1::users_client::UsersClient;
use api::users::v1alpha1::{GetUserRequest, CreateUserRequest, DeleteUserRequest, User};
use api::meta::v1alpha1::Metadata;
use api::{tonic, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://[::1]:9624", None)?;
    let mut uc = client.users();

    let request = tonic::Request::new(CreateUserRequest {
        user: Some(User {
            metadata: Some(Metadata {
                name: "root".into(),
            }),
        }),
    });

    let response = uc.create_user(request).await?;
    println!("CREATE USER={:?}", response.into_inner());

    let request = tonic::Request::new(GetUserRequest {
        name: "root".into(),
    });

    let response = uc.get_user(request).await?;

    println!("GET USER={:?}", response.into_inner());


    let request = tonic::Request::new(DeleteUserRequest {
        name: "root".into(),
    });

    let response = uc.delete_user(request).await?;

    println!("DELETE USER");

    Ok(())
}

