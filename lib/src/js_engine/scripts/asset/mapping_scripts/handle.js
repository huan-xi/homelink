// dev

const props = {
    "arming-mode": {
        siid: 3,
        piid: 1
    }
}
/*

homekit table
pub enum Value {
    StayArm = 0,
    AwayArm = 1,
    NightArm = 2,
    Disarm = 3,
    AlarmTriggered = 4,
}

*/

export default {
    // init func
    init() {

    },

    // on characteristic read
    async on_read() {
        //arming-mode
        // 0 - basic_arming AwayArm = 3,
        // 1 - home_arming StayArm = 0,
        // 2 - away_arming AwayArm = 1
        // 3 - sleep_arming NightArm = 2,
        let value = await dev.get_property(props["arming-mode"]);
        switch (value) {
            /// basic_arming
            case 0:
                ///
                return 3;
            // home_arming
            case 1:
                return 0;
            case 2:
                return 1;
            case 3:
                return 2;
            default:
                return 0;
        }
    },

    // on characteristic update
    on_update(old_vale: any, new_val: any) {

    },

    // on device event
    on_event(event: any): Promise<any> {

    },
}