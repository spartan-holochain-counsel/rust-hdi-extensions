import { AgentPubKey, HoloHash,
	 ActionHash, EntryHash }	from '@spartan-hc/holo-hash';
import {
    OptionType, VecType, MapType,
}					from '@whi/into-struct';


export const PostStruct = {
    "message":			String,
    "author":			AgentPubKey,

    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			Object,
};

export default {
    PostStruct,
};
