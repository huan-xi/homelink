import {core, primordials} from "ext:core/mod.js";

const {
    op_open_main_listener, op_accept_event,
    op_update_char_value,
    op_send_resp, op_device_set_property, op_device_read_property
} = core.ensureFastOps();

class MainChannel {
    constructor(rid) {
        this.rid = rid;
    }

    async accept_event() {
        // call op
        return await op_accept_event(this.rid);
    }

    /**
     *
     * @param msgId number
     * @param param {{type:string}}
     * @returns {Promise<void>}
     */
    async send_response(msgId, param) {
        return await op_send_resp(msgId, param);
    }
}

class Env {
    info;
    version;

    get_device(id) {

    }

    async open_main_listener() {
        let rid = await op_open_main_listener();
        return new MainChannel(rid);
    }
}

export class IotDevice {
    constructor(id) {
        this.id = id;
    }

    /**
     * set property
     * @param obj
     * @param value
     * @returns {Promise<void>}
     */
    async setProperty(obj, value) {
        const {siid, piid} = obj;
        if (value !== null && value !== undefined) {
            await op_device_set_property(this.id, siid, piid, value);
        }
    }

    async readProperty(obj) {
        const {siid, piid} = obj;
        return await op_device_read_property(this.id, siid, piid);
    }
}

export class Accessory {
    constructor(id) {
        this.id = id;
    }

    async updateCharacteristic(serviceTag, charTag, value) {
        op_update_char_value(this.id, serviceTag, charTag, value);
    }

}

const env = new Env();
// env.info = op_env_info();
env.info = "text env info"

globalThis.env = env;


globalThis.new_device = function (id) {
    return new IotDevice(id);
}
globalThis.new_accessory = function (id) {
    return new Accessory(id);
}