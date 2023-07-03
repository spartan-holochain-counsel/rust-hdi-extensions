use std::collections::BTreeMap;
use hdi::prelude::*;


//
// Post Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct PostEntry {
    pub message: String,
    pub author: AgentPubKey,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, rmpv::Value>,
}
