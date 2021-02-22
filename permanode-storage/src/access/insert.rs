use super::*;

impl<'a> Insert<'a, Bee<MessageId>, Bee<Message>> for Mainnet {
    fn get_request(
        &self,
        key: &Bee<MessageId>,
        value: &Bee<Message>,
    ) -> InsertRequest<Self, Bee<MessageId>, Bee<Message>> {
        let mut message_bytes = Vec::new();
        value.pack(&mut message_bytes).expect("Error occurred packing Message");
        let query = Query::new()
            .statement(&format!(
                "INSERT INTO {}.messages (message_id, message) VALUES (?, ?)",
                Self::name()
            ))
            .consistency(scylla_cql::Consistency::One)
            .value(key.to_string())
            .value(message_bytes)
            .build();

        let token = 1;

        InsertRequest::new(query, token, self)
    }
}

impl<'a> Insert<'a, Bee<MessageId>, Bee<MessageMetadata>> for Mainnet {
    fn get_request(
        &self,
        key: &Bee<MessageId>,
        value: &Bee<MessageMetadata>,
    ) -> InsertRequest<Self, Bee<MessageId>, Bee<MessageMetadata>> {
        let mut metadata_bytes = Vec::new();
        value
            .pack(&mut metadata_bytes)
            .expect("Error occurred packing MessageMetadata");
        let query = Query::new()
            .statement(&format!(
                "INSERT INTO {}.messages (message_id, message_metadata) VALUES (?, ?)",
                Self::name()
            ))
            .consistency(scylla_cql::Consistency::One)
            .value(key.to_string())
            .value(metadata_bytes)
            .build();

        let token = 1;

        InsertRequest::new(query, token, self)
    }
}

impl<'a> Insert<'a, Bee<MessageId>, (Bee<Message>, Bee<MessageMetadata>)> for Mainnet {
    fn get_request(
        &self,
        key: &Bee<MessageId>,
        (message, metadata): &(Bee<Message>, Bee<MessageMetadata>),
    ) -> InsertRequest<Self, Bee<MessageId>, (Bee<Message>, Bee<MessageMetadata>)> {
        let mut message_bytes = Vec::new();
        message
            .pack(&mut message_bytes)
            .expect("Error occurred packing Message");

        let mut metadata_bytes = Vec::new();
        metadata
            .pack(&mut metadata_bytes)
            .expect("Error occurred packing MessageMetadata");

        let query = Query::new()
            .statement(&format!(
                "INSERT INTO {}.messages (message_id, message_metadata) VALUES (?, ?)",
                Self::name()
            ))
            .consistency(scylla_cql::Consistency::One)
            .value(key.to_string())
            .value(metadata_bytes)
            .build();

        let token = 1;

        InsertRequest::new(query, token, self)
    }
}

impl<'a> Insert<'a, Bee<MilestoneIndex>, Bee<Milestone>> for Mainnet {
    fn get_request(
        &self,
        key: &Bee<MilestoneIndex>,
        value: &Bee<Milestone>,
    ) -> InsertRequest<Self, Bee<MilestoneIndex>, Bee<Milestone>> {
        let mut milestone_bytes = Vec::new();
        value
            .pack(&mut milestone_bytes)
            .expect("Error occurred packing Milestone");
        let query = Query::new()
            .statement(&format!(
                "INSERT INTO {}.milestones (milestone_index, milestone) VALUES (?, ?)",
                Self::name()
            ))
            .consistency(scylla_cql::Consistency::One)
            .value(key.to_string())
            .value(milestone_bytes)
            .build();

        let token = 1;

        InsertRequest::new(query, token, self)
    }
}

// impl_insert!(Mainnet: <(MessageId, MessageId), ()> -> { todo!() });
// impl_insert!(Mainnet: <(HashedIndex, MessageId), ()> -> { todo!() });
// impl_insert!(Mainnet: <OutputId, CreatedOutput> -> { todo!() });
// impl_insert!(Mainnet: <OutputId, ConsumedOutput> -> { todo!() });
// impl_insert!(Mainnet: <Unspent, ()> -> { todo!() });
// impl_insert!(Mainnet: <(Ed25519Address, OutputId), ()> -> { todo!() });
// impl_insert!(Mainnet: <(), LedgerIndex> -> { todo!() });
// impl_insert!(Mainnet: <(), SnapshotInfo> -> { todo!() });
// impl_insert!(Mainnet: <SolidEntryPoint, MilestoneIndex> -> { todo!() });
// impl_insert!(Mainnet: <MilestoneIndex, OutputDiff> -> { todo!() });
// impl_insert!(Mainnet: <Address, Balance> -> { todo!() });
// impl_insert!(Mainnet: <(MilestoneIndex, UnconfirmedMessage), ()> -> { todo!() });
