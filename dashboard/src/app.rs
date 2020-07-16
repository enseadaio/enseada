use yew::prelude::*;
use yew_router::router::Router;

use crate::component::Navbar;
use crate::router::AppRouter;

pub struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        App
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let render = Router::render(AppRouter::routes);

        let redirect = Router::redirect(AppRouter::not_found);

        html! {
            <div>
                <header>
                    <Navbar />
                </header>
                <main>
                    <section class="section">
                        <p></p>
                    </section>
                    <Router<AppRouter>
                        render = render
                        redirect = redirect
                    />
                </main>
            </div>
        }
    }
}
