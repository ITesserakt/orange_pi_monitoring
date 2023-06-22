#![forbid(unsafe_code)]

pub mod average_plot;
pub mod bar;
pub mod client;
pub mod model;
pub mod view;

use std::time::Duration;
use ybc::*;
use yew::{function_component, html, Html};

use crate::model::Model;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <Navbar fixed={NavbarFixed::Top} padded=true classes="has-background-light py-1" navbrand={html!{
                <NavbarItem>
                    <Title>{"Orange Pi monitoring service"}</Title>
                </NavbarItem>
            }} navstart={html!{}} navend={html!{}}/>
            <Hero fixed_nav=true size={HeroSize::FullheightWithNavbar} body={html!{
                <Container fluid={true}>
                    <Model update_interval={Duration::from_secs(1)}/>
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
