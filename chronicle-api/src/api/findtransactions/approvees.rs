use chronicle_cql::{
    frame::decoder::{
        Decoder,
        ColumnDecoder,
        Frame,
    },
    frame::query::Query,
    frame::header::{self, Header},
    frame::consistency::Consistency,
    frame::queryflags::{SKIP_METADATA, VALUES},
    compression::compression::UNCOMPRESSED,
    rows,
};
use std::collections::HashSet;

// ----------- decoding scope -----------

rows!(
    rows: Hashes {hashes: HashSet<String>},
    row: Row(
        Hash
    ),
    column_decoder: ApproveesDecoder
);

trait Rows {
    fn decode(self) -> Self;
    fn finalize(self) -> HashSet<String>;
}

impl Rows for Hashes {
    fn decode(mut self) -> Self {
        while let Some(_) = self.next() {};
        self
    }
    fn finalize(self) -> HashSet<String> {
        self.hashes
    }
}
// implementation to decode the columns in order to form the hash eventually
impl ApproveesDecoder for Hash {
    fn decode_column(start: usize, length: i32, acc: &mut Hashes) {
        // decode transaction hash
        let hash = String::decode(
            &acc.buffer()[start..], length as usize
        );
        // insert hash into hashset
        acc.hashes.insert(hash);
    }
}

// ----------- encoding scope -----------

/// Create a query frame to lookup for tx-hashes in the edge table using an approve
pub fn query(approve: String) -> Vec<u8> {
    let Query(payload) = Query::new()
        .version()
        .flags(header::IGNORE)
        .stream(0)
        .opcode()
        .length()
        .statement(
            "SELECT tx FROM tangle.edge WHERE vertex = ? AND kind in ['trunk','branch']"
        )
        .consistency(Consistency::One)
        .query_flags(SKIP_METADATA | VALUES)
        .value_count(1)
        .value(approve)
        .build(UNCOMPRESSED);
    payload
}