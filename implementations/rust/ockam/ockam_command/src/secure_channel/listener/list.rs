use clap::Args;
use colorful::Colorful;
use miette::miette;
use tokio::sync::Mutex;
use tokio::try_join;

use ockam::Context;
use ockam_api::nodes::models::secure_channel::{
    ListSecureChannelListenerResponse, ShowSecureChannelListenerResponse,
};
use ockam_api::nodes::BackgroundNode;
use ockam_api::route_to_multiaddr;
use ockam_core::route;

use crate::node::NodeOpts;
use crate::output::Output;
use crate::terminal::OckamColor;
use crate::util::api;
use crate::util::node_rpc;
use crate::{docs, CommandGlobalOpts};

const LONG_ABOUT: &str = include_str!("./static/list/long_about.txt");
const PREVIEW_TAG: &str = include_str!("../../static/preview_tag.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/list/after_long_help.txt");

/// List Secure Channel Listeners
#[derive(Args, Clone, Debug)]
#[command(
arg_required_else_help = true,
long_about = docs::about(LONG_ABOUT),
before_help = docs::before_help(PREVIEW_TAG),
after_long_help = docs::after_help(AFTER_LONG_HELP),
)]
pub struct ListCommand {
    /// Node of which secure listeners shall be listed
    #[command(flatten)]
    node_opts: NodeOpts,
}

impl ListCommand {
    pub fn run(self, opts: CommandGlobalOpts) {
        node_rpc(rpc, (opts, self));
    }
}

async fn rpc(ctx: Context, (opts, cmd): (CommandGlobalOpts, ListCommand)) -> miette::Result<()> {
    run_impl(&ctx, opts, cmd).await
}

async fn run_impl(ctx: &Context, opts: CommandGlobalOpts, cmd: ListCommand) -> miette::Result<()> {
    let node = BackgroundNode::create(ctx, &opts.state, &cmd.node_opts.at_node).await?;
    let is_finished: Mutex<bool> = Mutex::new(false);

    let get_listeners = async {
        let listeners: ListSecureChannelListenerResponse =
            node.ask(ctx, api::list_secure_channel_listener()).await?;
        *is_finished.lock().await = true;
        Ok(listeners)
    };

    let output_messages = vec![format!(
        "Listing secure channel listeners on {}...\n",
        node.node_name().color(OckamColor::PrimaryResource.color())
    )];

    let progress_output = opts
        .terminal
        .progress_output(&output_messages, &is_finished);

    let (secure_channel_listeners, _) = try_join!(get_listeners, progress_output)?;

    let list = opts.terminal.build_list(
        &secure_channel_listeners.list,
        &format!("Secure Channel Listeners at Node {}", node.node_name()),
        &format!(
            "No secure channel listeners found at node {}.",
            node.node_name()
        ),
    )?;
    opts.terminal.stdout().plain(list).write_line()?;

    Ok(())
}

impl Output for ShowSecureChannelListenerResponse {
    fn output(&self) -> crate::Result<String> {
        let addr = {
            let channel_route = &route![self.addr.clone()];
            let channel_multiaddr = route_to_multiaddr(channel_route).ok_or(miette!(
                "Failed to convert route {channel_route} to multi-address"
            ))?;
            channel_multiaddr.to_string()
        }
        .color(OckamColor::PrimaryResource.color());

        Ok(format!("Address {addr}"))
    }
}
