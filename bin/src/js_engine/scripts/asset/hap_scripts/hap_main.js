import handle from "../mapping_scripts/handle";

const accept_event = async (aid) => {
    // let event = await accept_event(aid);
    // 调用op 获取时间

    return event;
}

while (true) {
    //await event
    let event = await accept_event(context.aid);
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