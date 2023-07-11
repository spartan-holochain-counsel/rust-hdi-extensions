mod macros;

use core::convert::{ TryFrom, TryInto };
use hdi::prelude::{
    must_get_action,
    must_get_entry,
    must_get_valid_record,
    ExternResult, WasmError, WasmErrorInner,
    Deserialize, Serialize, SerializedBytesError,
    ActionHash, EntryHash, ExternalHash, AnyDhtHash, AnyLinkableHash,
    Record, Action, Entry, EntryCreationAction, ActionType,
    SignedActionHashed, EntryHashed,
    AppEntryDef, ScopedEntryDefIndex,
    EntryType, EntryTypesHelper,
    // Action Types
    Dna, AgentValidationPkg, InitZomesComplete,
    CreateLink, DeleteLink, OpenChain, CloseChain,
    Create, Update, Delete,
};
use hdi::prelude::holo_hash::AnyLinkableHashPrimitive;


// Prefix association train alternatives for 'must_'
// - sure, precise, exact, rigid, strict, pure, hard, assured, ensured, secure, guaranteed
//
// In the end, 'pure_get' or any other prefixes for 'get' seemed to long.  'summon' implies that the
// thing is not present but it exists somewhere else.
/// Alias for [`must_get_valid_record`]
pub fn summon_valid_record(action_hash: ActionHash) -> ExternResult<Record> {
    must_get_valid_record(action_hash)
}

/// Alias for [`must_get_action`]
pub fn summon_action(action_hash: ActionHash) -> ExternResult<SignedActionHashed> {
    must_get_action(action_hash)
}

/// Alias for [`must_get_entry`]
pub fn summon_entry(entry_hash: EntryHash) -> ExternResult<EntryHashed> {
    must_get_entry(entry_hash)
}


//
// Error Handling
//
/// Replace [`SerializedBytesError::Deserialize`] in [`WasmErrorInner::Serialize`] with [`WasmErrorInner::Guest`]
fn convert_deserialize_error(error: WasmError) -> WasmError {
    match error {
        WasmError { error: WasmErrorInner::Serialize(SerializedBytesError::Deserialize(msg)), .. } =>
            guest_error!(
                format!("Could not deserialize any-linkable address to expected type: {}", msg )
            ),
        err => err,
    }
}


//
// Tracing Actions
//
/// Collect the chain of evolutions backwards
pub fn trace_origin(action_address: &ActionHash) -> ExternResult<Vec<(ActionHash, Action)>> {
    let mut history = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
        let record = summon_valid_record( addr )?;

        next_addr = match record.action() {
            Action::Update(update) => Some(update.original_action_address.to_owned()),
            Action::Create(_) => None,
            _ => return Err(guest_error!(format!("Wrong action type '{}'", record.action().action_type() )))?,
        };

        history.push( (record.signed_action.hashed.hash, record.signed_action.hashed.content) );
    }

    Ok( history )
}


/// Get the last item in a [`trace_origin`] result
///
/// This should always be a [`Create`] action.
pub fn trace_origin_root(action_address: &ActionHash) -> ExternResult<(ActionHash, Action)> {
    Ok( trace_origin( action_address )?.last().unwrap().to_owned() )
}


//
// Entry Struct
//
/// Methods for getting scoped-type info from an entry struct
pub trait ScopedTypeConnector<T,U>
where
    ScopedEntryDefIndex: for<'a> TryFrom<&'a T, Error = WasmError>,
{
    /// Get this entry's corresponding unit enum
    fn unit() -> U;
    /// Get this entry's [`AppEntryDef`]
    fn app_entry_def() -> AppEntryDef;
    /// Check if a [`Record`]'s entry type matches this entry
    fn check_record_entry_type(record: &Record) -> bool;
    /// Deserialize a [`Record`]'s [`Entry`] into this struct
    fn try_from_record(record: &Record) -> Result<Self, Self::Error>
    where
        Self: TryFrom<Record>;
    /// Wrap this entry in the corresponding entry type enum
    fn to_input(&self) -> T;
}

/// Defines [`ScopedTypeConnector`] methods for an entry type
///
/// Rule patterns
/// - #1 - `<unit enum>::<unit name>, <types enum>( <entry struct> )`
/// - #2 - `<unit enum>::<unit name>, <types enum>, <entry struct>`
///
/// ##### Example: Basic Usage
/// ```ignore
/// use hdi::prelude::*;
/// use hdi_extensions::*;
///
/// #[hdk_entry_helper]
/// struct PostEntry {
///     pub message: String,
/// }
///
/// #[hdk_entry_defs]
/// #[unit_enum(EntryTypesUnit)]
/// pub enum EntryTypes {
///     #[entry_def]
///     Post(PostEntry),
/// }
///
/// scoped_type_connector!(
///     EntryTypesUnit::Post,
///     EntryTypes::Post( PostEntry )
/// );
/// ```
#[macro_export]
macro_rules! scoped_type_connector {
    ($units:ident::$unit_name:ident, $types:ident::$name:ident( $entry:ident ) ) => {
        scoped_type_connector!( $units::$unit_name, $types::$name, $entry );
    };
    ($units:ident::$unit_name:ident, $types:ident::$name:ident, $entry:ident ) => {
        impl ScopedTypeConnector<$types,$units> for $entry {

            fn unit() -> $units {
                $units::$unit_name
            }

            fn app_entry_def () -> AppEntryDef {
                // We know this is always defined because the hdi macros (hdk_entry_defs, unit_enum)
                // ensure that there will be a corresponding entry type for each unit.
                AppEntryDef::try_from( Self::unit() ).unwrap()
            }

            fn check_record_entry_type (record: &Record) -> bool {
                match EntryCreationAction::try_from( record.action().to_owned() ) {
                    Ok(creation_action) => match creation_action.entry_type() {
                        EntryType::App(aed) => Self::app_entry_def() == *aed,
                        _ => false,
                    },
                    _ => false,
                }
            }

            /// This "try from" checks the record's `EntryType` to make sure it matches the expected
            /// `AppEntryDef` and then uses the official `TryFrom<Record>`.
            fn try_from_record (record: &Record) -> Result<Self, WasmError> {
                let creation_action = EntryCreationAction::try_from( record.action().to_owned() )
                    .map_err(|_| hdi_extensions::guest_error!(
                        format!("ID does not belong to a Creation Action")
                    ))?;

                if let EntryType::App(aed) = creation_action.entry_type() {
                    if Self::app_entry_def() == *aed {
                        Ok( record.to_owned().try_into()? )
                    } else {
                        Err(hdi_extensions::guest_error!(
                            format!("Entry def mismatch: {:?} != {:?}", Self::app_entry_def(), aed )
                        ))
                    }
                } else {
                    Err(hdi_extensions::guest_error!(
                        format!("Action type ({}) does not contain an entry", ActionType::from(record.action()) )
                    ))
                }
            }

            fn to_input(&self) -> $types {
                $types::$name(self.clone())
            }
        }
    };
}


//
// HoloHash Extentions
//
/// Extend [`AnyLinkableHash`] transformations
pub trait AnyLinkableHashTransformer : Sized {
    /// Automatically determine correct type from a string
    fn try_from_string(input: &str) -> ExternResult<Self>;
    /// Expect hash type to be an [`ActionHash`] or error
    fn must_be_action_hash(&self) -> ExternResult<ActionHash>;
    /// Expect hash type to be an [`EntryHash`] or error
    fn must_be_entry_hash(&self) -> ExternResult<EntryHash>;
}

impl AnyLinkableHashTransformer for AnyLinkableHash {
    fn try_from_string(input: &str) -> ExternResult<Self> {
        let action_result = ActionHash::try_from( input.to_string() );
        let entry_result = EntryHash::try_from( input.to_string() );
        let external_result = ExternalHash::try_from( input.to_string() );

        Ok(
            match (action_result.is_ok(), entry_result.is_ok(), external_result.is_ok()) {
                (true, false, false) => action_result.unwrap().into(),
                (false, true, false) => entry_result.unwrap().into(),
                (false, false, true) => external_result.unwrap().into(),
                (false, false, false) => Err(guest_error!(
                    format!("String '{}' must be an Action or Entry hash", input )
                ))?,
                _ => Err(guest_error!(
                    format!("String '{}' matched multiple hash types; this should not be possible", input )
                ))?,
            }
        )
    }

    fn must_be_action_hash(&self) -> ExternResult<ActionHash> {
        match self.to_owned().into_action_hash() {
            Some(hash) => Ok( hash ),
            None => Err(guest_error!(
                format!("Any-linkable hash must be an action hash; not '{}'", self )
            ))?,
        }
    }

    fn must_be_entry_hash(&self) -> ExternResult<EntryHash> {
        match self.to_owned().into_entry_hash() {
            Some(hash) => Ok( hash ),
            None => Err(guest_error!(
                format!("Any-linkable hash must be an entry hash; not '{}'", self )
            ))?,
        }
    }
}

/// Extend [`AnyDhtHash`] transformations
pub trait AnyDhtHashTransformer : Sized {
    /// Automatically determine correct type from a string
    fn try_from_string(input: &str) -> ExternResult<Self>;
}

impl AnyDhtHashTransformer for AnyDhtHash {
    fn try_from_string(input: &str) -> ExternResult<Self> {
        let action_result = ActionHash::try_from( input.to_string() );
        let entry_result = EntryHash::try_from( input.to_string() );

        Ok(
            match (action_result.is_ok(), entry_result.is_ok()) {
                (true, false) => action_result.unwrap().into(),
                (false, true) => entry_result.unwrap().into(),
                (false, false) => Err(guest_error!(
                    format!("String '{}' must be an Action or Entry hash", input )
                ))?,
                (true, true) => Err(guest_error!(
                    format!("String '{}' matched Action and Entry hash; this should not be possible", input )
                ))?,
            }
        )
    }
}


//
// Advanced "get" Methods
//
/// Get and deserialize the given address into the expected app entry struct
///
/// **NOTE:** *This will only verify the deserialization of an app entry, it does not validate the
/// app entry def*
///
/// ##### Example: Basic Usage
/// ```
/// # use hdi::prelude::*;
/// # use hdi_extensions::*;
///
/// # #[hdk_entry_helper]
/// # struct PostEntry {
/// #     pub message: String,
/// # }
///
/// fn test(any_linkable_hash: AnyLinkableHash) -> ExternResult<()> {
///     let post : PostEntry = summon_app_entry( &any_linkable_hash )?;
///     Ok(())
/// }
/// ```
pub fn summon_app_entry<T,E>(addr: &AnyLinkableHash) -> ExternResult<T>
where
    T: TryFrom<Record, Error = E> + TryFrom<Entry, Error = E>,
    E: std::fmt::Debug,
    WasmError: From<E>,
{
    match addr.to_owned().into_primitive() {
        AnyLinkableHashPrimitive::Action(action_hash) => Ok(
            summon_valid_record( action_hash )?.try_into()
                .map_err(|error| convert_deserialize_error( WasmError::from(error) ) )?
        ),
        AnyLinkableHashPrimitive::Entry(entry_hash) => Ok(
            summon_entry( entry_hash )?.content.try_into()
                .map_err(|error| convert_deserialize_error( WasmError::from(error) ) )?
        ),
        AnyLinkableHashPrimitive::External(external_hash) => Err(guest_error!(
            format!("Cannot get an entry from any-linkable external hash ({})", external_hash )
        ))?,
    }
}

/// Check that the given address can deserialize to the expected app entry struct
///
/// **NOTE:** *This will only verify the deserialization of an app entry, it does not validate the
/// app entry def*
///
/// ##### Example: Basic Usage
/// ```
/// # use hdi::prelude::*;
/// # use hdi_extensions::*;
///
/// # #[hdk_entry_helper]
/// # struct PostEntry {
/// #     pub message: String,
/// # }
///
/// fn test(any_linkable_hash: AnyLinkableHash) -> ExternResult<()> {
///     verify_app_entry_struct::<PostEntry>( &any_linkable_hash )?;
///     Ok(())
/// }
/// ```
pub fn verify_app_entry_struct<T>(addr: &AnyLinkableHash) -> ExternResult<()>
where
    T: TryFrom<Record, Error = WasmError> + TryFrom<Entry, Error = WasmError>,
{
    let _ : T = summon_app_entry( addr )?;

    Ok(())
}

/// Get a [`Record`] expecting it to have a specific [`ActionType`]
pub fn summon_record_type(
    action_addr: &ActionHash,
    action_type: &ActionType
) -> ExternResult<Record> {
    let record = summon_valid_record( action_addr.to_owned() )?;

    if record.action().action_type() != *action_type {
        Err(guest_error!(format!("Action address ({}) is not a {} record", action_addr, action_type )))?
    }

    Ok( record )
}

macro_rules! get_action_type {
    ( $action_type:ident, $fn_name:ident ) => {
        #[doc = concat!("Get an action address expecting it to be a [`Action::", stringify!($action_type), "`]")]
        pub fn $fn_name(
            action_addr: &ActionHash,
        ) -> ExternResult<$action_type> {
            match summon_record_type( action_addr, &ActionType::$action_type )?.signed_action.hashed.content {
                Action::$action_type( action_inner ) => Ok( action_inner ),
                _ => Err(guest_error!("This should be unreachable".to_string())),
            }
        }
    };
}

get_action_type!( Dna, summon_dna_action );
get_action_type!( AgentValidationPkg, summon_agent_validation_pkg_action );
get_action_type!( InitZomesComplete, summon_init_zomes_complete_action );
get_action_type!( CreateLink, summon_create_link_action );
get_action_type!( DeleteLink, summon_delete_link_action );
get_action_type!( OpenChain, summon_open_chain_action );
get_action_type!( CloseChain, summon_close_chain_action );
get_action_type!( Create, summon_create_action );
get_action_type!( Update, summon_update_action );
get_action_type!( Delete, summon_delete_action );


/// Get an action address that is expected to be a [`EntryCreationAction`]
pub fn summon_creation_action(action_addr: &ActionHash) -> ExternResult<EntryCreationAction> {
    let create_record = summon_valid_record( action_addr.to_owned() )?;
    match create_record.signed_action.hashed.content {
        Action::Create(create) => Ok( create.into() ),
        Action::Update(update) => Ok( update.into() ),
        _ => Err(guest_error!(format!("Action address ({}) is not a create action", action_addr ))),
    }
}

/// Extend [`Action`] transformations
pub trait ActionTransformer : Sized {
    /// Get the app entry that this action points to
    ///
    /// ##### Example: Basic Usage
    /// ```ignore
    /// use hdi::prelude::*;
    /// use hdi_extensions::*;
    ///
    /// #[hdk_entry_helper]
    /// struct PostEntry {
    ///     pub message: String,
    /// }
    ///
    /// #[hdk_entry_defs]
    /// #[unit_enum(EntryTypesUnit)]
    /// pub enum EntryTypes {
    ///     #[entry_def]
    ///     Post(PostEntry),
    /// }
    ///
    /// fn test(addr: ActionHash) -> ExternResult<()> {
    ///     let action = summon_action( addr )?.hashed.content;
    ///     let entry_type = action.summon_app_entry<EntryTypes>()?;
    ///     Ok(())
    /// }
    /// ```
    fn summon_app_entry<ET>(&self) -> ExternResult<ET>
    where
        ET: EntryTypesHelper,
        WasmError: From<<ET as EntryTypesHelper>::Error>;
}

impl ActionTransformer for Action {
    fn summon_app_entry<ET>(&self) -> ExternResult<ET>
    where
        ET: EntryTypesHelper,
        WasmError: From<<ET as EntryTypesHelper>::Error>,
    {
        let action = EntryCreationAction::try_from( self.to_owned() )
            .map_err(|err| guest_error!(err.0))?;
        let entry_def = detect_app_entry_def( &action )?;
        let entry = summon_entry( action.entry_hash().to_owned() )?.content;

        ET::deserialize_from_type(
            entry_def.zome_index.clone(),
            entry_def.entry_index.clone(),
            &entry,
        )?.ok_or(guest_error!(
            format!("No match for entry def ({:?}) in expected entry types", entry_def )
        ))
    }
}


//
// EntryTypesHelper extensions
//
/// Detect the [`AppEntryDef`] from a given [`Action`]
pub fn detect_app_entry_def<A>(action: &A) -> ExternResult<AppEntryDef>
where
    A: Into<EntryCreationAction> + Clone,
{
    let action : EntryCreationAction = action.to_owned().into();
    match action.entry_type().to_owned() {
        EntryType::App(app_entry_def) => Ok( app_entry_def ),
        entry_type => Err(guest_error!(
            format!("Expected an app entry type; not {:?}", entry_type )
        )),
    }
}

/// Detect the entry types unit from a given [`Action`]
pub fn detect_app_entry_unit<ETU,A>(action: &A) -> ExternResult<ETU>
where
    ETU: TryFrom<ScopedEntryDefIndex, Error = WasmError>,
    A: Into<EntryCreationAction> + Clone,
{
    let action : EntryCreationAction = action.to_owned().into();
    let entry_def = detect_app_entry_def( &action )?;
    ETU::try_from(ScopedEntryDefIndex {
        zome_index: entry_def.zome_index,
        zome_type: entry_def.entry_index,
    })
}


//
// Standard Inputs
//
/// Input for getting links based on direction (ignoring type/tag)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LinkDirectionInput {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
}
