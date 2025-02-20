import { invoke } from "@tauri-apps/api/core";
import { Controller } from "./models";

export const startController = async (): Promise<Controller> => {
    try {
        let res = await invoke('start_controller', {});
        const controller = res as Controller;
        return controller;
    } catch (e) {
        console.error(e);
        throw e;
    }
};

export const stopController = async (): Promise<Controller> => {
    try {
        let res = await invoke('stop_controller', {});
        const controller = res as Controller;
        return controller;
    } catch (e) {
        console.error(e);
        throw e;
    }
};


export const postController = async () => {
    try {
        let controller: Controller = {
            'is_running': false,
            'exchange': {
                'name': 'bybit',
                'key': 'key',
                'secret': 'secret',
                'passphrase': 'passphrase'
            },
            'board': {
                'side': 'bid',
                'hight': 100,
                'low': 100,
                'size': 100
            },
            'order': {
                'symbol': 'BTCUSDT',
                'size': 100,
                'side': 'buy',
                'is_post_only': true,

                'tick_size': 0.01,

                'interval_sec': 5,

            }
        };


        let res = await invoke('post_controller', { 'value': controller });

        console.log(res);
    } catch (e) {
        console.error(e);
    }
};

export const getController = async () => {
    try {
        let res = await invoke('get_controller', {});

        console.log(res);
    } catch (e) {
        console.error(e);
    }
};


export const putController = async () => {
    try {
        let controller: Controller = {
            'is_running': false,
            'exchange': {
                'name': 'bybit',
                'key': 'key',
                'secret': 'secret',
                'passphrase': 'passphrase'
            },
            'board': {
                'side': 'bid',
                'hight': 100,
                'low': 100,
                'size': 100
            },
            'order': {
                'symbol': 'BTCUSDT',
                'size': 100,
                'side': 'buy',
                'is_post_only': true,

                'tick_size': 0.01,

                'interval_sec': 5,
            }
        };


        let res = await invoke('put_controller', { 'value': controller });

        console.log(res);
    } catch (e) {
        console.error(e);
    }
};

export const deleteController = async () => {
    try {
        let res = await invoke('delete_controller', {});

        console.log(res);
    } catch (e) {
        console.error(e);
    }
};
