use crate::average_plot::AverageF32Plot;
use crate::bar::Bar;
use crate::model::Message::{ChangeDestination, Connect};
use crate::model::Model;
use common::monitoring::{MemoryResponse, NetworkResponse};
use smallvec::SmallVec;
use std::sync::Arc;
use tonic::Status;
use ybc::*;
use yew::{function_component, html, Context, Html};

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
                <Tile ctx={TileCtx::Child} classes="box has-background-danger has-text-light"
                        size={TileSize::Eight}>
                    <Title classes="has-text-light">{"Failed to connect to service"}</Title>
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

fn cpu_view(temperature: &SmallVec<[f32; 60]>, usage: &Vec<f32>) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Cpu properties"}</Title>
            <Tile>
                <Tile ctx={TileCtx::Parent}>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <Subtitle>{"Current temperature"}</Subtitle>
                        <AverageF32Plot y_data={[temperature.to_vec()]}
                                     x_name={"Time, sec"}
                                     y_name={"Temperature, °C"}
                                     main_series_name={"Temperature"}
                                     main_series_color={(0x37, 0x6c, 0x5f)}
                                     avg_series_name={"Average temperature"}
                                     avg_series_color={(0x47, 0x89, 0x78)}
                        />
                    </Tile>
                </Tile>
                <Tile ctx={TileCtx::Parent} size={TileSize::Six}>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <Subtitle>{"Usage"}</Subtitle>
                        {usage.iter().map(|&x| html!{ <Bar fill={x} class="my-2"/> }).collect::<Html>()}
                    </Tile>
                </Tile>
            </Tile>
        </Tile>
    }
}

fn memory_view(memory: &SmallVec<[MemoryResponse; 60]>) -> Html {
    let as_megabytes = |f: fn(&MemoryResponse) -> u64| {
        move |memory: &MemoryResponse| f(memory) as f64 / 1024.0 / 1024.0
    };
    let free = memory
        .iter()
        .map(as_megabytes(|x| x.free))
        .map(|x| x as f32)
        .collect::<Vec<_>>();
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Memory properties"}</Title>
            <Tile>
                <Tile ctx={TileCtx::Parent} size={TileSize::Four} vertical=true>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <p>
                            <Subtitle>{"Total memory"}</Subtitle>
                            {format!("{:.2}", memory.last().map(as_megabytes(|x| x.total)).unwrap_or(0.0))}
                            {"MB"}
                        </p>
                    </Tile>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <p>
                            <Subtitle>{"Used memory"}</Subtitle>
                            {format!("{:.2}", memory.last().map(as_megabytes(|x| x.used)).unwrap_or(0.0))}
                            {"MB"}
                        </p>
                    </Tile>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <p>
                            <Subtitle>{"Available memory"}</Subtitle>
                            {format!("{:.2}", memory.last().map(as_megabytes(|x| x.available)).unwrap_or(0.0))}
                            {"MB"}
                        </p>
                    </Tile>
                </Tile>
                <Tile ctx={TileCtx::Parent}>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <Subtitle>{"Free memory"}</Subtitle>
                        <AverageF32Plot y_data={[free]}
                                     x_name={"Time, sec"}
                                     y_name={"Free memory, MB"}
                                     main_series_name={"Free memory"}
                                     main_series_color={(0x82, 0x35, 0x4F)}
                                     avg_series_name={"Average free memory"}
                                     avg_series_color={(0x65, 0x29, 0x3D)}
                        />
                    </Tile>
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
                    <Table fullwidth=true scrollable=true>
                        <thead>
                            <tr>
                                <th></th>
                                {network.interfaces.iter().map(|x| html!{
                                    <th>{x.name.clone()}</th>
                                }).collect::<Html>()}
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <th>{"Bytes in"}</th>
                                {network.interfaces.iter().map(|x| html!{
                                    <td>{x.bytes_in}</td>
                                }).collect::<Html>()}
                            </tr>
                            <tr>
                                <th>{"Bytes out"}</th>
                                {network.interfaces.iter().map(|x| html!{
                                    <td>{x.bytes_out}</td>
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
    memory: &SmallVec<[MemoryResponse; 60]>,
    connected_to: Arc<String>,
) -> Html {
    html! {
        <Tile ctx={TileCtx::Ancestor} classes="is-justify-content-center mt-1">
            <Tile vertical=true size={TileSize::Ten}>
                { connected_to_view(ctx, connected_to) }
                { cpu_view(temperature_window, usage) }
                { memory_view(memory) }
                { network_view(network) }
            </Tile>
        </Tile>
    }
}
