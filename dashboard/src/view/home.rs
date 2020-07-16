use yew::prelude::*;
use yew_router::components::RouterAnchor;

use crate::router::AppRouter;

pub struct HomeView;

impl Component for HomeView {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        HomeView
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <section class="section">
                <div class="columns is-multiline">
                    <div class="column">
                        <RouterAnchor<AppRouter> route=AppRouter::Containers>
                            <div class="box has-background-oci">
                                <article class="media">
                                    <div class="media-left">
                                        <figure class="image is-64x64 has-background-white">
                                            <img src="/static/images/oci-logo.png" alt="OCI Logo" />
                                        </figure>
                                    </div>
                                    <div class="media-content">
                                        <div class="">
                                            <h2 class="title has-text-white">{"Container images"}</h2>
                                            <h3 class="subtitle has-text-white">
                                                {"0 available"}
                                            </h3>
                                        </div>
                                    </div>
                                </article>
                            </div>
                        </RouterAnchor<AppRouter>>
                    </div>
                    <div class="column">
                        <RouterAnchor<AppRouter> route=AppRouter::Maven>
                            <div class="box has-background-maven">
                                <article class="media">
                                    <div class="media-left">
                                        <figure class="image is-64x64 has-background-white">
                                            <img class="is-rounded" src="/static/images/maven-logo.png" alt="Maven Logo" />
                                        </figure>
                                    </div>
                                    <div class="media-content">
                                        <div>
                                            <h2 class="title has-text-white">{"Maven packages"}</h2>
                                            <h3 class="subtitle has-text-white">
                                                {"0 available"}
                                            </h3>
                                        </div>
                                    </div>
                                </article>
                            </div>
                        </RouterAnchor<AppRouter>>
                    </div>
                </div>
            </section>
        }
    }
}
