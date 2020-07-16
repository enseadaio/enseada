use yew::prelude::*;
use yew_router::components::RouterAnchor;

use crate::router::AppRouter;

pub struct Navbar {
    link: ComponentLink<Self>,
    menu_open: bool,
}

pub enum Msg {
    ToggleMenu,
}

impl Component for Navbar {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Navbar {
            link,
            menu_open: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMenu => {
                self.menu_open = !self.menu_open;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let active_class = if self.menu_open { "is-active" } else { "" };
        let toggle_cb = self.link.callback(|_| Msg::ToggleMenu);
        html! {
        <nav class="navbar">
            <div class="navbar-brand">
                <RouterAnchor<AppRouter> route=AppRouter::Home classes="navbar-item">
                    <img src="/static/images/enseada-logo.svg" width="112" height="28" />
                </RouterAnchor<AppRouter>>

                <a role="button"
                   class=format!("navbar-burger burger {}", active_class)
                   aria-label="menu"
                   aria-expanded="false"
                   onclick=toggle_cb>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div class=format!("navbar-menu {}", active_class)>
                <div class="navbar-start">
                    <RouterAnchor<AppRouter> route=AppRouter::Home classes="navbar-item">
                    {"Home"}
                    </RouterAnchor<AppRouter>>

                    <div class="navbar-item has-dropdown is-hoverable">
                        <a class="navbar-link">{"Help"}</a>

                        <div class="navbar-dropdown">
                            <a class="navbar-item" href="https://docs.enseada.io" target="_blank">
                                {"Documentation"}
                            </a>
                            <a class="navbar-item" href="https://github.com/enseadaio/enseada/issues/new/choose"
                               target="_blank">
                                {"Report an issue"}
                            </a>
                            <hr class="navbar-divider" />
                            <a class="navbar-item" href="#">
                                {"About"}
                            </a>
                        </div>
                    </div>
                </div>

                <div class="navbar-end">

                </div>
            </div>
        </nav>
        }
    }
}
