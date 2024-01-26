// import env from "ext:deno_env/env.js";
// console.log("fetch", await fetch("https://www.baidu.com"));
console.log("main module exec")

/**
 * 注册的模块
 * @type {Map<number, {
 *     onCharRead: (event: { chId: number; method: string; params: any; }) => Promise<any>;
 *     onCharUpdate: () => Promise<any>;
 *     onDeviceEvent: () => Promise<any>;
 * }>}
 */
const moduleMap = new Map()
/**
 * 设备通道,模型中存在onDeviceEvent方法
 * @type {Map<string, number[]>}
 */
const devChMap = new Map();

const eventHandlers = {
    /**
     *
     * @param mainChannel
     * @param msgId
     * @param event {{url:string,chId:number}}
     * @returns {Promise<any>}
     */
    async executeSideModule(mainChannel, msgId, event) {
        let chId = event.chId;
        let hapModule = moduleMap.get(chId);
        if (!hapModule) {
            hapModule = await import(event.url);
            moduleMap.set(chId, hapModule);
        }
        // await mainChannel.send_response(msgId, {type: "ExecuteModuleResp", chId: chId})
        await mainChannel.send_response(msgId, {type: "ExecuteModuleResp", chId: String(chId)})
    },

    /**
     *
     * @param mainChannel
     * @param msgId
     * @param event {{chId:number,devId:string}}
     * @returns {Promise<void>}
     */
    async bindDeviceModule(mainChannel, msgId, event) {
        let chId = event.chId;
        let devId = event.devId;
        let chs = devChMap.get(devId);
        if (!chs) {
            chs = [];
            devChMap.set(devId, chs);
        }
        chs.push(chId);
        await mainChannel.send_response(msgId, {type: "BindDeviceModuleResp"})
    },
    /**
     *
     * @param mainChannel
     * @param msgId
     * @param event {{chId:number,serviceTag:string,charTag:string}}
     * @returns {Promise<void>}
     */
    async onCharRead(mainChannel, msgId, event) {
        // read property
        let value = await moduleMap.get(event.chId).onCharRead(event.serviceTag, event.charTag);
        await mainChannel.send_response(msgId, {type: "CharReadResp", value: value})
    },
    async onCharUpdate(mainChannel, msgId, event) {
        // read property
        await moduleMap.get(event.chId).onCharUpdate(event.serviceTag, event.charTag, event.oldValue, event.newValue);
        await mainChannel.send_response(msgId, {type: "CharUpdateResp"})
    },
    /**
     * 设备事件,分发到对应的模块中去
     * @param mainChannel
     * @param msgId
     * @param event {{devId: string}}
     * @returns {Promise<void>}
     */
    async onDeviceEvent(mainChannel, msgId, event) {
        let chs = devChMap.get(event.devId);
        if (chs) {
            for (let ch of chs) {
                let module = moduleMap.get(ch);
                if (module["onDeviceEvent"]) {
                    try {
                        await module.onDeviceEvent(event);
                    } catch (e) {
                        console.error("onDeviceEvent error ch", e, ch)
                    }

                }
            }

        }
    }


}

function firstLetterToLower(str) {
    return str.charAt(0).toLowerCase() + str.slice(1);
}

async function main() {
    let ch = await env.open_main_listener();

    while (true) {
        let msg = await ch.accept_event();
        if (!msg) {
            break;
        }
        let [msgId, event] = msg;
        console.log(`msgId:${msgId}:event:`, event);
        try {
            let handler = eventHandlers[firstLetterToLower(event.type)];
            if (handler) {
                handler(ch, msgId, event).then(() => {
                    //handle success
                }).catch(e => {
                    console.error("event hande error", e)
                    ch.send_response(msgId, {type: "ErrorResp", error: e.message});
                });
                continue;
            }
            console.error("event handler not found")
        } catch (e) {
            console.error("handle event error", e)
            //   send error event
            try {
                await ch.send_response(msgId, {type: "ErrorResp", error: e.message});
            } catch (e) {
                console.error("send error event error", e)
            }
        }
    }
}

try {
    await main();
    console.log("main exit");
} catch (e) {
    console.error("e", e)
}