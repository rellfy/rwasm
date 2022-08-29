
export default class Rwasm {
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
     * @param {Object} [functions]
     * @param {Object} [importObject]
     * @param {string} [mainFuncName]
     */
    constructor(
        path,
        functions,
        importObject = {},
        mainFuncName = "main"
    ) {
        this.mainFuncName = mainFuncName;
        this.functions = Rwasm.mergeObjectsRecursively(this.rwasmFunctions, functions);
        this.listeners = {};
        this.initialise(path, importObject);
    }

    get importObject() {
        return {
            env: {
                upload_bytes: this.receiveBytes.bind(this),
                request_timeout: this.registerTimeout.bind(this),
                seconds_now: () => Date.now()/1000.0,
            }
        };
    }

    get rwasmFunctions() {
        return {
            console_log: (message) => {
                console.log(`[WASM] ${message}`);
            },
            console_error: (message) => {
                console.error(`[WASM] ${message}`);
            },
        };
    }

    /**
     * @param {Object} a
     * @param {Object} b
     */
    static mergeObjectsRecursively(a, b) {
        const c = a;
        for (let key in b) {
            if (typeof b[key] !== "object" || c[key] === null) {
                c[key] = b[key];
                continue;
            }
            if (typeof c[key] !== "object")
                c[key] = {};
            c[key] = Rwasm.mergeObjectsRecursively(c[key], b[key]);
        }
        return c;
    }

    /**
     * @param {string} path
     * @param {Object} importObject
     */
    async initialise(path, importObject) {
        let merged = Rwasm.mergeObjectsRecursively(this.importObject, importObject);
        const { instance } = await WebAssembly.instantiateStreaming(
            fetch(path),
            merged,
        );
        this.instance = instance;
        this.emit("load");
        this.instance.exports[this.mainFuncName]();
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

        try {
            return this.functions[funcName](string, bufferId);
        } catch (e) {
            throw new Error(`Could not execute requested procedure "${funcName}": ${e}`);
        }
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
        setTimeout(() => {
            this.instance.exports.trigger_timeout(id);
        }, millis);
    }
}
