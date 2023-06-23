#![forbid(unsafe_code)]

pub mod average_plot;
pub mod bar;
pub mod client;
pub mod model;
pub mod view;

use std::time::Duration;
use ybc::*;
use yew::{function_component, html, use_state, Callback, Html};

use crate::model::Model;

#[function_component(App)]
fn app() -> Html {
    let update_interval = use_state(|| Duration::from_secs(1));
    let onclick = |new_duration| {
        let update_interval = update_interval.clone();
        Callback::from(move |_| {
            update_interval.set(new_duration);
        })
    };

    html! {
        <>
            <Navbar fixed={NavbarFixed::Top} padded=true classes="has-background-light py-1" navbrand={html!{
                <NavbarItem>
                    <Title>{"Orange Pi monitoring service"}</Title>
                </NavbarItem>
            }} navstart={html!{}} navend={html!{
                <NavbarDropdown navlink={html!{<p>{"Update interval"}</p>}} right=true>
                    <NavbarItem>
                        <Button classes="is-inverted is-link" onclick={onclick(Duration::from_secs(1))}>
                            {"1 second"}
                        </Button>
                    </NavbarItem>
                    <NavbarItem>
                        <Button classes="is-inverted is-link" onclick={onclick(Duration::from_secs(5))}>
                            {"5 seconds"}
                        </Button>
                    </NavbarItem>
                    <NavbarItem>
                        <Button classes="is-inverted is-link" onclick={onclick(Duration::from_millis(100))}>
                            {"100 milliseconds"}
                        </Button>
                    </NavbarItem>
                </NavbarDropdown>
            }}/>
            <Hero fixed_nav=true size={HeroSize::FullheightWithNavbar} body={html!{
                <Container fluid={true}>
                    <Model update_interval={*update_interval}/>
                </Container>
            }}/>
            <Footer>
                <Container classes="has-text-centered">
                    <a href="https://github.com/ITesserakt">{"Nikitin Vladimir"}</a>
                    {" - Bauman Moscow state technical university"}
                </Container>
            </Footer>
        </>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn launch() {
    yew::Renderer::<App>::new().render();
}
