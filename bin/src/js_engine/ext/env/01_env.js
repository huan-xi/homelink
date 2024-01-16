import {core, primordials} from "ext:core/mod.js";

const {
    op_main_listen
} = core.ensureFastOps();

class MainChannel {


    async listen() {
        // call op
        await op_main_listen();
    }
}

class Env {
    info;
    version;

    get_device(id) {

    }

    get_main_channel() {
        return new MainChannel();
    }
}
class IotDevice {
    set_property(name, value) {
    }
}

const env = new Env();
// env.info = op_env_info();
env.info = "test"

globalThis.env = env;