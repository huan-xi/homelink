/// mapping prop js template
// let context = {}
//import
import handle from "./mapping_script/test_001.js"

while (true) {
    //await event
    let event = await accept_event();
    let context = event.context;
    switch (event) {
        case "on_read":
            await handle.on_read(context);
            break;
        case "on_update":
            await handle.on_update(context);
            break;
        default:
            break;
    }
}