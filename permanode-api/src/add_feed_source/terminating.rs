use super::*;

#[async_trait]
impl<H: LauncherSender<PermanodeBuilder<H>>> Terminating<PermanodeSender<H>> for AddFeedSource {
    async fn terminating(
        &mut self,
        status: Result<(), Need>,
        supervisor: &mut Option<PermanodeSender<H>>,
    ) -> Result<(), Need> {
        todo!()
    }
}
