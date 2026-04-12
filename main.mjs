/** WARNING: For demo only; WASI for node.js is unsafe for now **/

import { readFile } from "node:fs/promises";

import { WASI } from "node:wasi";

import { isrc2txt2tokens } from "./test.mjs";

async function main() {
	/** @type {string} */
	const wname = "./rs_jptxt2tokens4web.wasm";

	const wasi = new WASI({
		version: "preview1",
	});

	const pbuf = readFile(wname);

	const imports = wasi.getImportObject();

	/** @type {Promise<WebAssembly.Module>} */
	const pwmod = pbuf.then((buf) => WebAssembly.compile(buf));

	/** @type {Promise<WebAssembly.Instance>} */
	const pins = pwmod.then((wmod) => WebAssembly.instantiate(wmod, imports));

	const init2txt2tokens = isrc2txt2tokens(() => pins);

	/** @type {function(string): Promise<string>} */
	const txt2tokens = init2txt2tokens((ins) => wasi.start(ins));

	/** @type {Promise<string>} */
	const ptokens = txt2tokens("helo, wrld");

	return ptokens;
}

main().then(console.info).catch(console.error);
