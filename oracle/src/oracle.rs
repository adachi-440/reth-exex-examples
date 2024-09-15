use futures::FutureExt;
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::{error, info};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{exex::ExEx, network::Network};

/// The Oracle struct is a long running task that orchestrates discovery of new peers,
/// decoding data from chain events (ExEx) and gossiping it to peers.
pub(crate) struct Oracle<Node: FullNodeComponents> {
    /// The network task for this node.
    /// It is composed by a discovery task and a sub protocol RLPx task.
    network: Network,
    /// The execution extension task for this node.
    exex: ExEx<Node>,
}

impl<Node: FullNodeComponents> Oracle<Node> {
    pub(crate) async fn new(exex: ExEx<Node>, network: Network) -> eyre::Result<Self> {
        Ok(Self { exex, network })
    }
}

impl<Node: FullNodeComponents> Future for Oracle<Node> {
    type Output = eyre::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut();
        // Poll the network future until its drained
        loop {
            match this.network.poll_unpin(cx) {
                Poll::Ready(Ok(())) => {
                    info!("Discv5 task completed successfully");
                }
                Poll::Ready(Err(e)) => {
                    error!(?e, "Discv5 task encountered an error");
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => {
                    // Exit match and continue to poll exex
                    break;
                }
            }
        }

        // Poll the exex future until its drained
        loop {
            match this.exex.poll_unpin(cx)? {
                Poll::Ready(t) => t,
                Poll::Pending => return Poll::Pending,
            };
        }
    }
}
