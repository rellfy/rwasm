const path = "./target/wasm32-unknown-unknown/release/rwasm.wasm";

const importobject = {
    env: {
        console_log: (ptr, length) => {
            const bytes = new Uint8Array(instance.exports.memory.buffer, ptr, length);
            console.log(new TextDecoder().decode(bytes));
        }
    }
};

function initialiseWasm({ instance }) {
    window.instance = instance;
    instance.exports.main();
    
    multiplyExample(instance);
    dataUploadExample(instance);
}

function multiplyExample(instance) {
    let result = instance.exports.multiply(3, 4);
    console.log(`Multiplication result: ${result}`);
}

/**
 * Gets pointer to a buffer from Rust and writes
 * byte array [ 10, 20, 30, 40, 50 ] into Rust buffer.
 */
function dataUploadExample(instance) {
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
    // Inform WASM of the data length.
    instance.exports.handle_buffer(data.length);
}

WebAssembly.instantiateStreaming(fetch(path), importobject).then(initialiseWasm);