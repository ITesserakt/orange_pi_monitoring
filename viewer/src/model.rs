use crate::bytesize::ByteSizeExt;
use crate::client::RpcClient;
use crate::model::Message::*;
use crate::model::Model::*;
use crate::view::{failed_view, populated_view, unloaded_view};
use common::monitoring::{CpuResponse, MemoryResponse, NetworkResponse, Pack};
use futures::stream::AbortHandle;
use futures::{pin_mut, StreamExt};
use prokio::{spawn_local, Runtime};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tonic::{Response, Status};
use yew::{Component, Context, Html, Properties};

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
        cpu_temp_window: Option<VecDeque<f32>>,
        free_mem_window: VecDeque<f32>,
        usage: Vec<f32>,
        memory_response: MemoryResponse,
        network_response: NetworkResponse,
        common: Common,
    },
}

pub enum Message {
    Fail(Status),
    Populate(CpuResponse, NetworkResponse, MemoryResponse),
    Connect,
    ChangeDestination(String),
}

impl From<Result<Response<Pack>, Status>> for Message {
    fn from(value: Result<Response<Pack>, Status>) -> Self {
        match value {
            Ok(x) => {
                let Pack {
                    cpu,
                    network,
                    memory,
                } = x.into_inner();
                Populate(cpu, network, memory)
            }
            Err(e) => Fail(e),
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub update_interval: Duration,
    #[prop_or_default]
    pub auto_connect: bool,
    #[prop_or(60)]
    pub window_size: usize,
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
        ctx: &Context<Self>,
    ) -> (Option<Message>, bool) {
        match (msg, self as &mut Self) {
            (Populate(cpu, network, memory), Connected(common, active)) => {
                let mut mem_array: VecDeque<f32> = Default::default();
                mem_array.push_back(memory.free.to_bytes().as_megabytes());
                let cpu_array = cpu.temperature.map(|x| VecDeque::from([x]));

                *self = Populated {
                    active: active.clone(),
                    cpu_temp_window: cpu_array,
                    free_mem_window: mem_array,
                    usage: cpu.usage,
                    network_response: network,
                    common: common.clone(),
                    memory_response: memory,
                };
                (None, true)
            }
            (
                Populate(cpu, network, memory),
                Populated {
                    cpu_temp_window,
                    network_response,
                    free_mem_window,
                    memory_response,
                    usage,
                    ..
                },
            ) => {
                cpu_temp_window
                    .as_mut()
                    .zip(cpu.temperature)
                    .map(|(window, temp)| {
                        if window.len() == ctx.props().window_size {
                            window.pop_front();
                        }
                        window.push_back(temp);
                    });
                if free_mem_window.len() == ctx.props().window_size {
                    free_mem_window.pop_front();
                }
                free_mem_window.push_back(memory.free.to_bytes().as_megabytes());
                *usage = cpu.usage;
                *network_response = network;
                *memory_response = memory;

                (None, true)
            }
            (Populate(..), _) => (None, false),
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
                let handle = Model::run_monitoring(ctx, common);
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

    fn run_monitoring(ctx: &Context<Model>, common: &mut Common) -> AbortHandle {
        let client = RpcClient::new(common.connection_address.as_ref().clone());
        let (stream, handle) = client.connect();
        let upd_interval = ctx.props().update_interval;
        let _runtime = Runtime::builder().build().unwrap();
        let scope = ctx.link().clone();

        spawn_local(async move {
            pin_mut!(stream);
            while let Some(next) = stream.next().await {
                scope.send_message(next);
                prokio::time::sleep(upd_interval).await;
            }
        });

        handle
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

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        match self {
            Connected(common, active) | Populated { common, active, .. } => {
                active.abort();
                *active = Self::run_monitoring(ctx, common);
            }
            _ => {}
        };

        false
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
                free_mem_window,
                memory_response,
                common,
                ..
            } => populated_view(
                ctx,
                cpu_temp_window,
                usage,
                network_response,
                free_mem_window,
                memory_response,
                common.connection_address.clone(),
            ),
        }
    }
}
