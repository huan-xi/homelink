// import user handle
import {context} from "./context.js";


const props = {
    "arming-mode": {
        siid: 3,
        piid: 1
    }
}
/**
 * homekit 米家
 * 0 在家  1.
 * 1 离家  2
 * 2 睡眠  3
 * 3 停用  2
 *
 * @type {Map<number, number>}
 */
const mi_to_homekit_mapping = new Map([
    [1, 0],
    [2, 1],
    [3, 2],
    [4, 3]
]);

const reverseMapping = new Map([...mi_to_homekit_mapping].map(([key, value]) => [value, key]));

export const onCharRead = async (service_tag, char_tag) => {
    console.log("on_read", service_tag, char_tag);
    //    get_property(siid: number, piid: number): Promise<any>;
    let value = await context.dev.readProperty(props["arming-mode"]);
    console.log("onCharRead value", value, mi_to_homekit_mapping.get(value));
    return mi_to_homekit_mapping.get(value);

};
export const onCharUpdate = async (service_tag, char_tag, old_value, new_value) => {
    let val = reverseMapping.get(new_value);
    await context.dev.setProperty(props["arming-mode"], val);
    await context.accessory.updateCharacteristic("test1", "security-system-state.current", new_value);
}

