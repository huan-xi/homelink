declare interface Context {

}




// a handler to process characteristic read/write and device event
declare interface CharacteristicHandler {
    // init func
    init(): void;

    // on characteristic read
    on_read(): Promise<any>;

    // on characteristic update
    on_update(old_vale: any, new_val: any): Promise<any>;

    // on device event
    on_event(event: any): Promise<any>;

}