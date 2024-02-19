//import
import handle from "./handle.js"
import {context} from "./context.js";

if (handle.init) {
    handle.init(context);
}

console.log("mapping 1")

while (true) {
    //await event
    let event = await accept_event();
    let context = event.context;
    switch (event) {
        case "on_read":
            // call user handle
            await handle.on_read(context);
            break;
        case "on_update":
            await handle.on_update(context);
            break;
        default:
            break;
    }
}