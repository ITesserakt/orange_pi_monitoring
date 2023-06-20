#![forbid(unsafe_code)]

pub mod bar;
pub mod client;
pub mod model;

use std::time::Duration;
use yew::{function_component, html, Html};

use crate::model::Model;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <main class="wrapper">
                <nav class="uk-navbar-container uk-margin">
                    <div class="uk-container">
                        <div class="uk-navbar uk-navbar-left">
                            <a uk-icon="icon: menu" class="uk-navbar-item uk-navbar-toggle" href=""></a>
                            <ul class="uk-navbar-item">
                                <li class="uk-logo">{"Orange Pi monitoring service"}</li>
                            </ul>
                        </div>
                    </div>
                </nav>
                <div class="uk-container">
                    <Model update_interval={Duration::from_millis(1000)} auto_connect={false}/>
                </div>
                <div class="buffer"></div>
            </main>
            <footer class="uk-padding uk-position-z-index">
                <div class="uk-container">
                    <a href="https://github.com/ITesserakt">{"Nikitin Vladimir"}</a>
                    {" - Bauman Moscow state technical university"}
                </div>
            </footer>
        </>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn main() {
    yew::Renderer::<App>::new().render();
}
