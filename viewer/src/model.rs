use crate::average_plot::AveragePlot;
use crate::bar::Bar;
use crate::client::RpcClient;
use crate::model::Message::*;
use crate::model::Model::*;
use crate::view::{failed_view, populated_view, unloaded_view};
use common::monitoring::{CpuResponse, NetworkResponse, Pack};
use futures::stream::AbortHandle;
use futures::TryStreamExt;
use smallvec::SmallVec;
use std::sync::Arc;
use std::time::Duration;
use tonic::{Response, Status};
use ybc::*;
use yew::{function_component, html, Component, Context, Html, Properties};

#[derive(Clone)]
pub struct Common {
    connection_address: Arc<String>,
}

pub enum Model {
    Unloaded(Common),
    Connected(Common, AbortHandle),
    Failed(Common, Status),
    Populated {
        active: AbortHandle,
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

impl From<Result<Response<Pack>, Status>> for Message {
    fn from(value: Result<Response<Pack>, Status>) -> Self {
        match value {
            Ok(x) => {
                let Pack {
                    cpu: Some(cpu),
                    network: Some(network),
                } = x.into_inner() else { unreachable!() };
                Populate(cpu, network)
            }
            Err(e) => Fail(e),
        }
    }
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

#[allow(clippy::unnecessary_cast)]
impl Model {
    #[inline]
    fn handle_fail_message(
        &mut self,
        msg: Message,
        redraw: bool,
        _ctx: &Context<Self>,
    ) -> (Option<Message>, bool) {
        match (msg, self as &mut Self) {
            (Fail(e), Connected(common, active) | Populated { active, common, .. }) => {
                active.abort();
                *self = Failed(common.clone(), e);
                (None, true)
            }
            (Fail(_), _) => (None, false),
            (other, _) => (Some(other), redraw),
        }
    }

    #[inline]
    fn handle_populate_message(
        &mut self,
        msg: Message,
        redraw: bool,
        _ctx: &Context<Self>,
    ) -> (Option<Message>, bool) {
        match (msg, self as &mut Self) {
            (Populate(cpu, network), Connected(common, active)) => {
                let mut array: SmallVec<_> = Default::default();
                if let Some(t) = cpu.temperature {
                    array.push(t)
                }

                *self = Populated {
                    active: active.clone(),
                    cpu_temp_window: array,
                    usage: cpu.usage,
                    network_response: network,
                    common: common.clone(),
                };
                (None, true)
            }
            (
                Populate(cpu, network),
                Populated {
                    cpu_temp_window,
                    network_response,
                    usage,
                    ..
                },
            ) => {
                if cpu_temp_window.len() == cpu_temp_window.inline_size() {
                    cpu_temp_window.remove(0);
                }
                if let Some(t) = cpu.temperature {
                    cpu_temp_window.push(t);
                }
                *usage = cpu.usage;
                *network_response = network;

                (None, true)
            }
            (Populate(_, _), _) => (None, false),
            (other, _) => (Some(other), redraw),
        }
    }

    #[inline]
    fn handle_connect_message(
        &mut self,
        msg: Message,
        redraw: bool,
        ctx: &Context<Self>,
    ) -> (Option<Message>, bool) {
        match (msg, self as &mut Self) {
            (Connect, Unloaded(common)) => {
                let client = RpcClient::new(common.connection_address.as_ref().clone());
                let (stream, handle) = client.connect();
                let upd_interval = ctx.props().update_interval;
                ctx.link().send_stream(stream.and_then(move |x| async move {
                    prokio::time::sleep(upd_interval).await;
                    Ok(x)
                }));
                *self = Connected(common.clone(), handle);
                (None, true)
            }
            (Connect, Failed(common, _)) => {
                *self = Unloaded(common.clone());
                (None, true)
            }
            (Connect, Populated { common, active, .. }) => {
                active.abort();
                *self = Unloaded(common.clone());
                (None, true)
            }
            (Connect, _) => (None, false),
            (other, _) => (Some(other), redraw),
        }
    }

    #[inline]
    fn handle_change_message(
        &mut self,
        msg: Message,
        redraw: bool,
        _ctx: &Context<Self>,
    ) -> (Option<Message>, bool) {
        match (msg, self as &mut Self) {
            (ChangeDestination(destination), Unloaded(common)) => {
                *common = Common {
                    connection_address: Arc::new(destination),
                };
                (None, false)
            }
            (ChangeDestination(_), _) => (None, false),
            (other, _) => (Some(other), redraw),
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
        let handlers = [
            Model::handle_populate_message,
            Model::handle_connect_message,
            Model::handle_change_message,
            Model::handle_fail_message,
        ];
        let mut msg = Some(msg);
        let mut redraw = false;

        for handler in handlers {
            if let Some(msg_) = msg {
                let bundle = handler(self, msg_, redraw, ctx);
                msg = bundle.0;
                redraw = bundle.1;
            } else {
                return redraw;
            }
        }

        redraw
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self {
            Connected(common, _) => unloaded_view(ctx, common.connection_address.clone(), true),
            Unloaded(form) => unloaded_view(ctx, form.connection_address.clone(), false),
            Failed(_, e) => failed_view(e, ctx),
            Populated {
                cpu_temp_window,
                usage,
                network_response,
                common,
                ..
            } => populated_view(
                ctx,
                cpu_temp_window,
                usage,
                network_response,
                common.connection_address.clone(),
            ),
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
