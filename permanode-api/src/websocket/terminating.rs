use super::*;

#[async_trait]
impl<H: PermanodeAPIScope> Terminating<PermanodeAPISender<H>> for Websocket {
    async fn terminating(
        &mut self,
        _status: Result<(), Need>,
        supervisor: &mut Option<PermanodeAPISender<H>>,
    ) -> Result<(), Need> {
        self.service.update_status(ServiceStatus::Stopping);
        if let Some(ref mut supervisor) = supervisor {
            supervisor
                .send(PermanodeAPIEvent::Children(PermanodeAPIChild::Listener(
                    self.service.clone(),
                )))
                .map_err(|_| Need::Abort)
        } else {
            Err(Need::Abort)
        }
    }
}