use super::*;
use mpsc::unbounded_channel;
use permanode_storage::{
    access::{
        GetSelectRequest,
        MessageId,
        MessageMetadata,
        ReporterEvent,
        SelectRequest,
    },
    keyspaces::Mainnet,
    types::Bee,
};
use rocket::{
    get,
    response::content::Json,
};
use scylla::ring::Ring;
use serde::Serialize;
use std::{
    borrow::Cow,
    str::FromStr,
};
use tokio::sync::mpsc;

#[async_trait]
impl<H: LauncherSender<PermanodeBuilder<H>>> EventLoop<PermanodeSender<H>> for Listener {
    async fn event_loop(
        &mut self,
        _status: Result<(), Need>,
        supervisor: &mut Option<PermanodeSender<H>>,
    ) -> Result<(), Need> {
        self.service.update_status(ServiceStatus::Running);
        if let Some(ref mut supervisor) = supervisor {
            supervisor
                .send(PermanodeEvent::Children(PermanodeChild::Listener(self.service.clone())))
                .map_err(|_| Need::Abort)?;
        }
        rocket::ignite()
            .mount(
                "/",
                routes![
                    get_message,
                    get_message_metadata,
                    get_message_children,
                    get_message_by_index
                ],
            )
            .launch()
            .await
            .map_err(|_| Need::Abort)
    }
}

async fn query<'a, V: Serialize, S: Select<'a, K, V>, K>(
    request: SelectRequest<'a, S, K, V>,
) -> Result<Json<String>, Cow<'static, str>> {
    let (sender, mut inbox) = unbounded_channel::<Event>();
    let worker = Box::new(DecoderWorker(sender));

    let decoder = request.send_local(worker);

    while let Some(event) = inbox.recv().await {
        match event {
            Event::Response { giveload } => {
                let res = decoder.decode(giveload);
                if let Ok(Some(message)) = res {
                    return Ok(Json(serde_json::to_string(&message).unwrap()));
                }
            }
            Event::Error { kind } => return Err(kind.to_string().into()),
        }
    }

    Err("Failed to receive response!".into())
}

#[get("/messages/<message_id>")]
async fn get_message(message_id: String) -> Result<Json<String>, Cow<'static, str>> {
    let request = Mainnet.select::<Bee<Message>>(&MessageId::from_str(&message_id).unwrap().into());
    query(request).await
}

#[get("/messages/<message_id>/metadata")]
async fn get_message_metadata(message_id: String) -> Result<Json<String>, Cow<'static, str>> {
    let request = Mainnet.select::<Bee<MessageMetadata>>(&MessageId::from_str(&message_id).unwrap().into());
    query(request).await
}

#[get("/messages/<message_id>/children")]
async fn get_message_children(message_id: String) -> Result<Json<String>, Cow<'static, str>> {
    todo!()
}

#[get("/messages?<index>")]
async fn get_message_by_index(index: String) -> Result<Json<String>, Cow<'static, str>> {
    todo!()
}
