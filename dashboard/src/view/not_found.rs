use yew::prelude::*;
use yew::{Component, ComponentLink, Html};

pub struct NotFoundView {}

impl Component for NotFoundView {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        NotFoundView {}
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{"Page Not Found"}</h1>
            </div>
        }
    }
}
