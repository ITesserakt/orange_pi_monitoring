use crate::average_plot::AveragePlot;
use crate::bar::Bar;
use crate::model::Message::{ChangeDestination, Connect};
use crate::model::Model;
use common::monitoring::NetworkResponse;
use smallvec::SmallVec;
use std::sync::Arc;
use tonic::Status;
use ybc::*;
use yew::{function_component, html, Context, Html, Properties};

#[function_component]
pub(crate) fn ConnectedView() -> Html {
    html! {
        <Title>{"Connected, waiting for an update..."}</Title>
    }
}

pub(crate) fn unloaded_view(ctx: &Context<Model>, destination: Arc<String>, loading: bool) -> Html {
    let onclick = ctx.link().callback(|_| Connect);
    let oninput = ctx.link().callback(ChangeDestination);

    html! {
        <Tile ctx={TileCtx::Ancestor}>
            <Tile ctx={TileCtx::Parent} classes="is-justify-content-center">
                <Tile ctx={TileCtx::Child} classes="box" size={TileSize::Eight}>
                    <Title>{"Enter service address"}</Title>
                    <div class="is-flex">
                        <Input update={oninput} name="Destination"
                                {loading} disabled={loading}
                                value={destination.as_ref().clone()}
                                classes="mr-3"/>
                        <Button classes="is-primary"
                                {loading} disabled={loading} {onclick}>{"Connect"}</Button>
                    </div>
                </Tile>
            </Tile>
        </Tile>
    }
}

pub(crate) fn failed_view(status: &Status, ctx: &Context<Model>) -> Html {
    html! {
        <Tile ctx={TileCtx::Ancestor}>
            <Tile ctx={TileCtx::Parent} classes="is-justify-content-center">
                <Tile ctx={TileCtx::Child} classes="box has-background-warning"
                        size={TileSize::Eight}>
                    <Title>{"Failed to connect to service"}</Title>
                    <p class="mb-2">{status}</p>
                    <Button classes="is-primary" onclick={ctx.link().callback(|_| Connect)}>
                        {"Reconnect"}
                    </Button>
                </Tile>
            </Tile>
        </Tile>
    }
}

fn connected_to_view(ctx: &Context<Model>, connected_to: Arc<String>) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent}>
            <Tile ctx={TileCtx::Child}
                classes="box has-background-dark is-justify-content-space-between is-flex is-align-items-center">
                <p class="has-text-white">
                    {"Connected to "}
                    <a href={connected_to.as_ref().clone()}>{"device"}</a>
                    {" at "}{ connected_to.as_ref().clone() }
                </p>
                <Button onclick={ctx.link().callback(|_| Connect)} classes="is-danger">
                    {"Disconnect"}
                </Button>
            </Tile>
        </Tile>
    }
}

fn cpu_view(ctx: &Context<Model>, temperature: &SmallVec<[f32; 60]>, usage: &Vec<f32>) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Cpu properties"}</Title>
            <Tile ctx={TileCtx::Parent}>
                <Tile ctx={TileCtx::Child} classes="box">
                    <Title>{"Current temperature"}</Title>
                    <AveragePlot y_data={temperature.to_vec()}
                                 x_name={"Time, sec"}
                                 y_name={"Temperature, °C"}
                                 main_series_name={"Temperature"}
                                 main_series_color={(0x37, 0x6c, 0x5f)}
                                 avg_series_name={"Average temperature"}
                                 avg_series_color={(0x47, 0x89, 0x78)}
                    />
                </Tile>
                <Tile ctx={TileCtx::Child} classes="box">
                    <Title>{"Usage"}</Title>
                    {usage.iter().map(|&x| html!{ <Bar fill={x} class="my-2"/> }).collect::<Html>()}
                </Tile>
            </Tile>
        </Tile>
    }
}

fn network_view(network: &NetworkResponse) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Network properties"}</Title>
            <Tile ctx={TileCtx::Parent}>
                <Tile ctx={TileCtx::Child} classes="box">
                    <Table fullwidth=true>
                        <thead>
                            <tr>
                            {network.interfaces.iter().map(|x| html!{
                                <th>{x.name.clone()}</th>
                            }).collect::<Html>()}
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                {network.interfaces.iter().map(|x| html!{
                                    <td>{"Bytes in: "}{x.bytes_in}</td>
                                }).collect::<Html>()}
                            </tr>
                            <tr>
                                {network.interfaces.iter().map(|x| html!{
                                    <td>{"Bytes out: "}{x.bytes_out}</td>
                                }).collect::<Html>()}
                            </tr>
                        </tbody>
                    </Table>
                </Tile>
            </Tile>
        </Tile>
    }
}

pub(crate) fn populated_view(
    ctx: &Context<Model>,
    temperature_window: &SmallVec<[f32; 60]>,
    usage: &Vec<f32>,
    network: &NetworkResponse,
    connected_to: Arc<String>,
) -> Html {
    html! {
        <Tile ctx={TileCtx::Ancestor} classes="is-justify-content-center mt-1">
            <Tile vertical=true size={TileSize::Eight}>
                { connected_to_view(ctx, connected_to) }
                { cpu_view(ctx, temperature_window, usage) }
                { network_view(network) }
            </Tile>
        </Tile>
    }
}
