import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-model-dna", process.env.LOG_LEVEL );

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import msgpack				from '@msgpack/msgpack';
import json				from '@whi/json';
import { AgentPubKey, HoloHash,
	 ActionHash, EntryHash }	from '@whi/holo-hash';
import HolochainBackdrop		from '@whi/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;
import {
    intoStruct,
    OptionType, VecType, MapType,
}					from '@whi/into-struct';

// const why				= require('why-is-node-running');
import {
    expect_reject,
    linearSuite,
    createPostInput,
}					from '../utils.js';
import {
    PostStruct,
}					from './types.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const clients				= {};
const DNA_NAME				= "test_dna";
const MAIN_ZOME				= "basic_usage_csr";


let p1, p1_addr;


function basic_tests () {

    it("should create group via alice (A1)", async function () {
	let input			= createPostInput( clients.alice.cellAgent() );
	p1_addr				= new ActionHash(
	    await clients.alice.call( DNA_NAME, MAIN_ZOME, "create_post", input )
	);
	log.debug("Post ID: %s", p1_addr );

	expect( p1_addr		).to.be.a("ActionHash");
	expect( p1_addr		).to.have.length( 39 );

	p1				= intoStruct(
	    await clients.alice.call( DNA_NAME, MAIN_ZOME, "get_post", {
		"id": p1_addr,
	    }),
	    PostStruct
	);
	log.debug( json.debug( p1 ) );
    });

}


function error_tests () {
}


describe("HDI Extensions", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "trace",
    });

    before(async function () {
	this.timeout( 300_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		[DNA_NAME]:		TEST_DNA_PATH,
	    },
	});

	for ( let name in actors ) {
	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client		= clients[ name ]	= actors[ name ][ app_prefix ].client;
	    }
	}

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( DNA_NAME, MAIN_ZOME, "whoami", null, 300_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    linearSuite("Basic",	basic_tests );
    describe("Errors",		error_tests );

    after(async () => {
	await holochain.destroy();
    });

});
