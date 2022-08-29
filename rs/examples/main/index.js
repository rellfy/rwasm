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
        const messageAsBytes = RWASM.stringToUint8Array(message);
        // Write data to WASM buffer and return data length.
        return wasm.sendUint8Array(messageAsBytes, bufferId);
    }
};

const path = "/target/wasm32-unknown-unknown/release/examples/main.wasm";
const wasm = new RWASM(path, functions);

wasm.on("load", () => {
    multiplyExample(wasm);
    dataUploadExample(wasm);
});
