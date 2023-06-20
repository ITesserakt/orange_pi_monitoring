use common::monitoring::{CpuResponse, NetworkResponse};
use prokio::spawn_local;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tonic::{Request, Status};
use web_sys::HtmlInputElement;
use yew::{html, Component, Context, Html, InputEvent, Properties, TargetCast};
use yew_plotly::plotly::color::Rgb;
use yew_plotly::plotly::common::{DashType, Line, Mode, Title};
use yew_plotly::plotly::layout::Axis;
use yew_plotly::plotly::{Layout, Plot, Scatter};
use yew_plotly::Plotly;

use crate::bar::Bar;
use crate::client::RpcClient;
use crate::model::Message::*;
use crate::model::Model::*;

#[derive(Clone)]
pub struct Common {
    connection_address: Arc<String>,
}

pub enum Model {
    Unloaded(Common),
    Connected(Common, Arc<AtomicBool>),
    Failed(Common, Status),
    Populated {
        active: Arc<AtomicBool>,
        cpu_temp_window: SmallVec<[f32; 60]>,
        usage: Vec<f32>,
        network_response: NetworkResponse,
        common: Common,
    },
}

pub enum Message {
    Fail(Status),
    Populate(CpuResponse, NetworkResponse),
    Connect,
    ChangeDestination(String),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub update_interval: Duration,
    #[prop_or_default]
    pub auto_connect: bool,
}

impl Default for Model {
    fn default() -> Self {
        Unloaded {
            0: Common {
                connection_address: Arc::new("http://orangepi:50525".to_string()),
            },
        }
    }
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if ctx.props().auto_connect {
            let mut model = Model::default();
            model.update(ctx, Connect);
            model
        } else {
            Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[allow(clippy::unnecessary_cast)]
        match (msg, self as &mut Self) {
            (Fail(e), Connected(common, active) | Populated { active, common, .. }) => {
                active.store(false, Ordering::Release);
                *self = Failed(common.clone(), e)
            }
            (ChangeDestination(dst), Unloaded(_)) => {
                *self = Unloaded(Common {
                    connection_address: Arc::new(dst),
                })
            }
            (Populate(t, f), Connected(common, active)) => {
                let mut array: SmallVec<_> = Default::default();
                if let Some(t) = t.temperature {
                    array.push(t);
                }
                *self = Populated {
                    common: common.clone(),
                    active: active.clone(),
                    cpu_temp_window: array,
                    usage: t.usage,
                    network_response: f,
                }
            }
            (
                Populate(t, f),
                Populated {
                    cpu_temp_window,
                    usage,
                    network_response,
                    ..
                },
            ) => {
                if let Some(t) = t.temperature {
                    cpu_temp_window.push(t);
                }
                if cpu_temp_window.len() == cpu_temp_window.inline_size() {
                    cpu_temp_window.remove(0);
                }
                *usage = t.usage;
                *network_response = f;
            }
            (Connect, Unloaded(dst)) => {
                *self = Connected(dst.clone(), Model::connect(ctx, &dst.connection_address))
            }
            (Connect, Failed(common, _)) => *self = Unloaded(common.clone()),
            (Connect, Populated { common, .. }) => *self = Unloaded(common.clone()),
            _ => return false,
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self {
            Connected(..) => html! {
                <div class="uk-padding uk-card">
                    <h3>{"Connected, waiting for an update"}</h3>
                    <div class="uk-spinner"></div>
                </div>
            },
            Unloaded(form) => {
                let onclick = ctx.link().callback(|_| Connect);
                let oninput = ctx.link().callback(|e: InputEvent| {
                    let element: HtmlInputElement = e.target_unchecked_into();
                    ChangeDestination(element.value())
                });

                html! {
                    <div class="uk-card uk-padding">
                        <h3>{"Enter service URL"}</h3>
                        <div class="uk-flex">
                            <input class="uk-input" type="text" {oninput} value={form.connection_address.as_ref().clone()}/>
                            <button class="uk-button uk-button-primary" {onclick}>{"Connect"}</button>
                        </div>
                    </div>
                }
            }
            Failed(_, e) => html! {
                <div class="uk-card uk-padding">
                    <h3>{"Failed to connect to service"}</h3>
                    <p>{"Cannot retrieve state: "}{e}</p>
                    <button class="uk-button-primary uk-button"
                            onclick={ctx.link().callback(|_| Connect)}>{"Reconnect"}</button>
                </div>
            },
            Populated {
                active: _,
                cpu_temp_window,
                usage,
                network_response,
                common,
            } => html! {
                <div>
                    <div class="uk-card-secondary
                                uk-card-body 
                                uk-flex 
                                uk-flex-between 
                                uk-card-hover 
                                uk-flex-middle 
                                uk-padding-small">
                        <p class="uk-margin-remove-bottom">
                            {"Connected to "}
                            <a href={ common.connection_address.as_ref().clone() }>{"device"}</a>
                            {" at "}{ common.connection_address.as_ref().clone() }
                        </p>
                        <button onclick={ctx.link().callback(|_| Connect)}
                                class="uk-button-danger uk-padding-small">
                            {"Disconnect"}
                        </button>
                    </div>
                    <div class="uk-card-default uk-card-body uk-padding uk-margin uk-grid-item-match">
                        <h2 class="uk-card-title">{"CPU properties:"}</h2>
                        <div class="uk-grid uk-child-width-expand@s uk-grid-divider">
                            <div class="uk-card uk-card-body">
                                {"Current temperature: "}
                                {Model::draw_temp_chart(cpu_temp_window)}
                            </div>
                            <div class="uk-card uk-card-body">
                                <p>{"Usage: "}</p>
                                {usage.iter().map(|&x|
                                    html! { <Bar fill={x} class="uk-light uk-margin-small"/> }
                                ).collect::<Html>()}
                            </div>
                        </div>
                    </div>
                    <div class="uk-card-default uk-card-body uk-padding uk-margin">
                        <h2 class="uk-card-title">{"Available networks:"}</h2>
                        <table class="uk-table uk-table-divider">
                            <thead>
                                <tr>
                                {network_response.interfaces.iter().map(|x| html!{
                                    <th>{x.name.clone()}</th>
                                }).collect::<Html>()}
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    {network_response.interfaces.iter().map(|x| html!{
                                        <td>{"Bytes in: "}{x.bytes_in}</td>
                                    }).collect::<Html>()}
                                </tr>
                                <tr>
                                    {network_response.interfaces.iter().map(|x| html!{
                                        <td>{"Bytes out: "}{x.bytes_out}</td>
                                    }).collect::<Html>()}
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            },
        }
    }
}

impl Model {
    fn draw_temp_chart<const N: usize>(temp: &SmallVec<[f32; N]>) -> Html {
        let mut plot = Plot::new();
        let temp = temp.to_vec();
        let min = *temp.iter().min_by_key(|&&x| x as u32).unwrap();
        let max = *temp.iter().max_by_key(|&&x| x as u32).unwrap();
        let cumavg = temp
            .iter()
            .scan((0.0, 0), |(acc, cnt), x| {
                *cnt += 1;
                *acc += *x;
                Some(*acc / *cnt as f32)
            })
            .collect();

        let layout = Layout::new()
            .x_axis(Axis::new().title(Title::new("Time, sec")))
            .y_axis(
                Axis::new()
                    .title(Title::new("Temperature, °C"))
                    .range(vec![min - 5.0, max + 5.0]),
            )
            .show_legend(false);

        let series = Scatter::new((0..N).into_iter().collect(), temp)
            .mode(Mode::Lines)
            .name("Temperature")
            .line(
                Line::new()
                    .dash(DashType::Dot)
                    .color(Rgb::new(0x37, 0x6C, 0x5F)),
            );

        let avg_series = Scatter::new((0..N).into_iter().collect(), cumavg)
            .mode(Mode::Lines)
            .name("Average temperature")
            .line(Line::new().color(Rgb::new(0x47, 0x89, 0x78)));

        plot.add_trace(series);
        plot.add_trace(avg_series);
        plot.set_layout(layout);

        html! { <Plotly {plot}/> }
    }

    async fn monitoring_loop(client: &mut RpcClient) -> Message {
        let cpu = client.monitor_cpu(Request::new(())).await;
        let network = client.monitor_network(Request::new(())).await;

        match (cpu, network) {
            (Ok(cpu), Ok(network)) => Populate(cpu.into_inner(), network.into_inner()),
            (Err(e), _) | (_, Err(e)) => Fail(e),
        }
    }

    fn connect(ctx: &Context<Model>, destination: &str) -> Arc<AtomicBool> {
        let active = Arc::new(AtomicBool::new(true));
        let mut client = RpcClient::new(destination.to_owned());
        let upd_interval = ctx.props().update_interval;
        let scope = ctx.link().clone();
        let _runtime = prokio::Runtime::builder().build().unwrap();
        let active_signal = active.clone();
        let stream = async move {
            while active_signal.load(Ordering::Acquire) {
                let message = Model::monitoring_loop(&mut client).await;
                scope.send_message(message);
                prokio::time::sleep(upd_interval).await;
            }
        };

        spawn_local(stream);
        active
    }
}
