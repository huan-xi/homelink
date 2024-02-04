/**
 *
 * @type {{dev: IotDevice, aid: number,accessory}}
 */
const context = {
    aid: 1195342208509673472,
    dev_id: 3,

}

context.dev = new_device(context.dev_id);
context.accessory = new_accessory(context.aid);
export {
    context
}