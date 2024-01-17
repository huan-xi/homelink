declare interface IotDevice {

    /**
     * 获取属性
     * @param siid
     * @param piid
     */
    readProperty(siid: number, piid: number): Promise<any>;

    /**
     * 设置属性
     * @param ssid
     * @param value
     */

    setProperty(ssid: number, value: any): Promise<void>;
}

declare interface MainChannel {
    await(): Promise<boolean>;
}

declare interface HapAccessoryListener {
    accept_event(): Promise<void>;
}

declare interface Env {
    info: string;
    version: string;

    get_device(id: number): IotDevice | null;
}