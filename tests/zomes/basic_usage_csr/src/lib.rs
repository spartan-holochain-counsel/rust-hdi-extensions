
use hdk::prelude::*;
use hdi_extensions::{
    guest_error,
    ScopedTypeConnector,
};
use basic_usage::{
    PostEntry,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'{}' init", zome_info()?.name );
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    Ok( agent_info()? )
}


#[hdk_extern]
pub fn create_post(post: PostEntry) -> ExternResult<ActionHash> {
    debug!("Creating new post entry: {:#?}", post );
    let action_hash = create_entry( post.to_input() )?;

    Ok( action_hash )
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetEntityInput {
    pub id: ActionHash,
}

#[hdk_extern]
pub fn get_post(input: GetEntityInput) -> ExternResult<PostEntry> {
    debug!("Get latest post entry: {:#?}", input.id );
    let record = get( input.id.clone(), GetOptions::latest() )?
        .ok_or(guest_error!(format!("Record not found: {}", input.id )))?;

    Ok( PostEntry::try_from_record( &record )? )
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateEntryInput<T> {
    pub base: ActionHash,
    pub entry: T,
}

#[hdk_extern]
pub fn update_post(input: UpdateEntryInput<PostEntry>) -> ExternResult<ActionHash> {
    debug!("Update post action: {}", input.base );
    let action_hash = update_entry( input.base, input.entry.to_input() )?;

    Ok( action_hash )
}
