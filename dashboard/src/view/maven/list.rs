use yew::prelude::*;
use yew_router::components::RouterAnchor;

use crate::router::AppRouter;

pub struct MavenRepoListView {}

impl Component for MavenRepoListView {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        MavenRepoListView {}
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
                <div class="level">
                    <div class="level-left">
                        <RouterAnchor<AppRouter> route=AppRouter::Home classes="level-item button">
                            {"Back"}
                        </RouterAnchor<AppRouter>>
                    </div>
                </div>
                <h1 class="title">{"Maven"}</h1>
            </section>
        }
    }
}
