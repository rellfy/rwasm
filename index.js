"use strict";

class WASM {

    /**
     * @type {Object}
     */
    functions;
    /**
     * @type {WebAssembly.Instance}
     */
    instance;
    /**
     * @type {string}
     */
    mainFuncName;
    /**
     * Event listeners.
     */
    listeners;

    /**
     * @param {string} path
     * @param {Object} functions
     * @param {string} [mainFuncName]
     */
    constructor(path, functions, mainFuncName = "main") {
        this.mainFuncName = mainFuncName;
        this.functions = functions;
        this.listeners = {};
        this.initialise(path);

    }

    get importObject() {
        return {
            env: {
                upload_bytes: this.receiveBytes.bind(this)
            }
        };
    }

    /**
     * @param {string} path
     */
    async initialise(path) {
        const { instance } = await WebAssembly.instantiateStreaming(
            fetch(path),
            this.importObject
        );
        this.instance = instance;
        this.instance.exports[this.mainFuncName]();
        this.emit("load");
    }

    /**
     * @param {string} event
     * @param {Object} [data]
     */
    emit(event, data) {
        for (let key in this.listeners) {
            if (key !== event)
                continue;

            this.listeners[event].forEach(func => {
                func(data);
            });
        }
    }

    /**
     * @param {string} event
     * @param {function} callback
     */
    on(event, callback) {
        if (this.listeners[event] == null)
            this.listeners[event] = [];

        this.listeners[event].push(callback);
    }

    /**
     * Returns a buffer from WASM under a pointer.
     * @param {number} pointer
     * @param  {number} length
     * @returns {Uint8Array}
     */
    getDataFromPointer(pointer, length) {
        return new Uint8Array(this.instance.exports.memory.buffer, pointer, length);
    }

    /**
     * Returns a string from WASM under a pointer.
     * @param {number} pointer
     * @param  {number} length
     * @returns {string}
     */
    getTextFromPointer(pointer, length) {
        const bytes = this.getDataFromPointer(pointer, length);
        return new TextDecoder().decode(bytes);
    }

    /**
     * Performs RPC call from WASM with function name and data from pointer.
     * Return value of RPC must be length of data written to WASM buffer (if any), or undefined.
     * @param {number} pointer
     * @param  {number} length
     * @returns {number | undefined}
     */
    receiveBytes(pointer, length) {
        let string = this.getTextFromPointer(pointer, length);
        let funcNameSize = 0;

        for (let i = 0; i < string.length; i++) {
            funcNameSize++;

            if (string[i] === "\0")
                break;
        }

        let funcName = string.substr(0, funcNameSize - 1);
        string = string.substr(funcNameSize, string.length - funcNameSize);

        if (string.length === 0)
            string = null;

        return this.functions[funcName](string);
    }
}

/**
 * Request a multiplication to be performed in WASM.
 * @param {WebAssembly.Instance} instance
 */
function multiplyExample({ instance }) {
    let result = instance.exports.multiply(3, 4);
    console.log(`[JS] Multiplication result: ${result}`);
}

/**
 * Gets pointer to a buffer from WASM and writes
 * byte array [ 10, 20, 30, 40, 50 ] into WASM buffer.
 * @param {WebAssembly.Instance} instance
 */
function dataUploadExample({ instance }) {
    // Get pointer.
    let bufferPointer =  instance.exports.get_buffer_pointer();
    // Example byte array.
    let data = new Uint8Array([ 10, 20, 30, 40, 50 ]);
    // Create new array before writing to WASM memory buffer.
    const u8 = new Uint8Array(instance.exports.memory.buffer, bufferPointer, data.length);
    // Write data to WASM memory.
    for (let i = 0; i < data.length; i++) {
        u8[i] = data[i];
    }
    // Inform WASM of the data length. Since this information wasn't requested by WASM,
    // it is of course not possible to return a length of the data here -- therefore a specific
    // function is used for this task (this could be further abstracted by passing both an id
    // and the data length, then mapping the ID to a function from WASM).
    instance.exports.handle_data_upload_example(data.length);
}

/**
 * Objects of functions which can be called from WASM through RPCs (see function receiveBytes).
 * All inputs are strings (which can be converted back to an Uint8Array if needed).
 * @type {{console_log: functions.console_log}}
 */
const functions = {
    console_log: (message) => {
        console.log(`[WASM] ${message}`);
    }
};

const path = "./target/wasm32-unknown-unknown/release/rwasm.wasm";
const wasm = new WASM(path, functions);

wasm.on("load", () => {
    multiplyExample(wasm);
    dataUploadExample(wasm);
});
