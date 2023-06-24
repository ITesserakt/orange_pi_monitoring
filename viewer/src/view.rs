use crate::average_plot::AverageF32Plot;
use crate::bar::Bar;
use crate::bytesize::{ByteSize, ByteSizeExt};
use crate::model::Message::{ChangeDestination, Connect};
use crate::model::Model;
use common::monitoring::{MemoryResponse, NetworkResponse};
use std::collections::VecDeque;
use std::sync::Arc;
use tonic::Status;
use ybc::*;
use yew::{function_component, html, AttrValue, Context, Html, Properties};

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

fn cpu_view(temperature: &Option<VecDeque<f32>>, usage: &Vec<f32>) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Cpu properties"}</Title>
            <Tile>
                <Tile ctx={TileCtx::Parent}>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <Subtitle>{"Current temperature"}</Subtitle>
                        {match temperature {
                            Some(temperature) => html!{
                                <AverageF32Plot y_data={[temperature.iter().cloned().collect::<Vec<_>>()]}
                                     x_name={"Time, sec"}
                                     y_name={"Temperature, °C"}
                                     main_series_name={"Temperature"}
                                     main_series_color={(0x37, 0x6c, 0x5f)}
                                     avg_series_name={"Average temperature"}
                                     avg_series_color={(0x47, 0x89, 0x78)}
                                />},
                            None => html!{ <p>{"Unavailable on current device"}</p> }
                        }}
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

#[derive(Properties, PartialEq, Clone)]
struct ParameterToPlotProps {
    name: AttrValue,
    bytes: ByteSize,
}

#[function_component]
fn ParameterToPlot(props: &ParameterToPlotProps) -> Html {
    html! {
        <Tile ctx={TileCtx::Child} classes="box is-relative">
            <Subtitle>{&props.name}</Subtitle>
            { props.bytes.to_string() }
            <Button classes="is-primary m-3 is-corner-right is-swap">
                <Icon classes="mdi mdi-swap-horizontal is-size-5"/>
            </Button>
        </Tile>
    }
}

fn memory_view(free_memory: &VecDeque<f32>, memory: &MemoryResponse) -> Html {
    html! {
        <Tile ctx={TileCtx::Parent} classes="is-flex is-flex-direction-column">
            <Title>{"Memory properties"}</Title>
            <Tile>
                <Tile size={TileSize::Four}>
                    <Tile ctx={TileCtx::Parent} vertical=true>
                        <ParameterToPlot name={"Total memory"} bytes={memory.total.to_bytes()}/>
                        <ParameterToPlot name={"Used memory"} bytes={memory.used.to_bytes()}/>
                        <ParameterToPlot name={"Available memory"} bytes={memory.available.to_bytes()}/>
                    </Tile>
                    <Tile ctx={TileCtx::Parent} vertical=true>
                        <ParameterToPlot name={"Total swap memory"} bytes={memory.swap.total.to_bytes()}/>
                        <ParameterToPlot name={"Used swap memory"} bytes={memory.swap.used.to_bytes()}/>
                        <ParameterToPlot name={"Free swap memory"} bytes={memory.swap.free.to_bytes()}/>
                    </Tile>
                </Tile>
                <Tile ctx={TileCtx::Parent}>
                    <Tile ctx={TileCtx::Child} classes="box">
                        <Subtitle>{"Free memory"}</Subtitle>
                        <AverageF32Plot y_data={[free_memory.iter().cloned().collect::<Vec<_>>()]}
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
    temperature_window: &Option<VecDeque<f32>>,
    usage: &Vec<f32>,
    network: &NetworkResponse,
    free_memory: &VecDeque<f32>,
    memory: &MemoryResponse,
    connected_to: Arc<String>,
) -> Html {
    html! {
        <Tile ctx={TileCtx::Ancestor} classes="is-justify-content-center mt-1">
            <Tile vertical=true size={TileSize::Ten}>
                { connected_to_view(ctx, connected_to) }
                { cpu_view(temperature_window, usage) }
                { memory_view(free_memory, memory) }
                { network_view(network) }
            </Tile>
        </Tile>
    }
}
