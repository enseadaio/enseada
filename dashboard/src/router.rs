use yew::prelude::*;
use yew_router::Switch;

use crate::view::*;
use yew_router::route::Route;

#[derive(Clone, Debug, Switch)]
pub enum AppRouter {
    #[to = "/containers"]
    Containers,
    #[to = "/maven"]
    Maven,
    #[to = "/"]
    Home,
    #[to = "/not-found"]
    NotFound,
}

impl AppRouter {
    pub fn routes(self) -> Html {
        log::debug!("Rendering route {:?}", self);
        match self {
            AppRouter::Home => html! {<HomeView/>},
            AppRouter::Containers => html! {<containers::ContainerListView/>},
            AppRouter::Maven => html! {<maven::MavenRepoListView/>},
            AppRouter::NotFound => html! {<NotFoundView/>},
        }
    }

    pub fn not_found(route: Route) -> Self {
        log::warn!("Route {} not found!", route.route);
        AppRouter::NotFound
    }
}
