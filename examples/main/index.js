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
                upload_bytes: this.receiveBytes.bind(this),
                request_timeout: this.registerTimeout.bind(this),
                now: () => Date.now()/1000.0,
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

        let bufferId = null;

        if (funcName.includes(".")) {
            bufferId = parseInt(funcName.substring(funcName.lastIndexOf(".") + 1));
            funcName = funcName.substring(0, funcName.lastIndexOf("."));
        }

        return this.functions[funcName](string, bufferId);
    }

    /**
     * Sends a byte array to a specific buffer.
     * @param array
     * @param bufferPointer
     * @returns {number}
     */
    sendUint8ArrayToBuffer(array, bufferPointer) {
        // Construct buffer inside WASM.
        const u8 = new Uint8Array(
            this.instance.exports.memory.buffer,
            bufferPointer,
            array.length
        );
        // Write data.
        for (let i = 0; i < array.length; i++) {
            u8[i] = array[i];
        }
        // Return data length to be uploaded to WASM.
        return array.length;
    }

    /**
     * Sends a byte array to a buffer.
     * @param {Uint8Array} array
     * @param {number} [id]
     * @returns {number}
     */
    sendUint8Array(array, id = 0) {
        return this.sendUint8ArrayToBuffer(array, this.instance.exports.get_buffer_pointer(id));
    }

    /**
     * Converts a string to an Uint8Array.
     * @param {string} string
     * @returns {Uint8Array}
     */
    static stringToUint8Array(string) {
        const array = new Uint8Array(string.length);
        for (let i in string) {
            array[i] = string.charCodeAt(i);
        }
        return array;
    }

    /**
     * @param {number} id
     * @param {number} millis
     */
    registerTimeout(id, millis) {
        console.log("timeout " + id + " for " + millis + "ms");
        setTimeout(() => {
            this.instance.exports.trigger_timeout(id);
        }, millis);
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
    // Get pointer for buffer 0.
    let bufferPointer =  instance.exports.get_buffer_pointer(0);
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
 */
const functions = {
    console_log: (message) => {
        console.log(`[WASM] ${message}`);
    },
    console_error: (message) => {
        console.error(message);
    },
    // Example: transform a message to upper case and send back to WASM.
    // When data is requested, the buffer id to write to is given with the message.
    request_data_example: (message, bufferId) => {
        message = message.toUpperCase();
        const messageAsBytes = WASM.stringToUint8Array(message);
        // Write data to WASM buffer and return data length.
        return wasm.sendUint8Array(messageAsBytes, bufferId);
    }
};

const path = "/target/wasm32-unknown-unknown/release/examples/main.wasm";
const wasm = new WASM(path, functions);

wasm.on("load", () => {
    multiplyExample(wasm);
    dataUploadExample(wasm);
});
