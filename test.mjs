/** @typedef {function(): Promise<WebAssembly.Instance>} InstanceSourceIO */

/** @typedef {object} ExportShape
 *  @property {WebAssembly.Memory} memory
 *  @property {function(): number} input_txt_ptr_e
 *  @property {function(): number} output_txt_ptr_e
 *  @property {function(number): number} txt2tokens2json2page_e
 */

/** @type {function(string, WebAssembly.Instance): Promise<string>} */
const txt2tokens = (txt, instance) => {
	/** @type {ExportShape} */
	const exports = /** @type {any} */ (instance.exports);

	const { input_txt_ptr_e, output_txt_ptr_e, txt2tokens2json2page_e } =
		exports || {};

	/** @type {number} */
	const iptr = input_txt_ptr_e();

	/** @type {number} */
	const bufsz = 65536;

	/** @type {WebAssembly.ExportValue} */
	const memory = exports.memory;

	/** @type {Uint8Array<ArrayBuffer>} */
	const iview = new Uint8Array(memory.buffer, iptr, bufsz);

	/** @type {TextEncoder} */
	const enc = new TextEncoder();

	/** @type {TextEncoderEncodeIntoResult} */
	const encodeResult = enc.encodeInto(txt, iview);

	if (encodeResult?.written === undefined) {
		return Promise.reject(new Error("unable to encode into"));
	}

	/** @type {number} */
	const input_size = encodeResult?.written;

	/** @type {number} */
	const output_size = txt2tokens2json2page_e(input_size);

	/** @type {number} */
	const optr = output_txt_ptr_e();

	/** @type {Uint8Array<ArrayBuffer>} */
	const oview = new Uint8Array(memory.buffer, optr, output_size);

	/** @type {TextDecoder} */
	const dec = new TextDecoder("utf-8");

	/** @type {string} */
	const ostr = dec.decode(oview);

	return Promise.resolve(ostr);
};

/** @typedef {function(string): Promise<string>} TxtToTokens */

/** @typedef {function(WebAssembly.Instance): void} Initializer */

/** @type {function(InstanceSourceIO): function(Initializer): TxtToTokens} */
const isrc2txt2tokens = (isrc) => (init) => (txt) => {
	return isrc().then((ins) => {
		init(ins);
		return txt2tokens(txt, ins);
	});
};

export { isrc2txt2tokens };
