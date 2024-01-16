declare interface IotDevice {


    /**
     * 获取属性
     * @param siid
     * @param piid
     */
    get_property(siid: number, piid: number): Promise<any>;

    /**
     * 设置属性
     * @param ssid
     * @param value
     */

    set_property(ssid: number, value: any): Promise<void>;
}

declare interface MainChannel {
    await(): Promise<boolean>;
}

declare interface Env {
    info: string;
    version: string;

    get_device(id: number): IotDevice | null;
}