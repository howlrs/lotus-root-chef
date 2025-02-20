import { useState } from "react";
import { Button, Drawer, Spin } from "antd";
import { invoke } from "@tauri-apps/api/core";

interface Log {
    level: string,
    message: string,
    timestamp: string,
};

const Logger = () => {
    const [open, setOpen] = useState(false);
    const [logger, setLogger] = useState<Log[]>([]);
    const [loading, setLoading] = useState(false);

    const fetchLogger = async () => {
        setLoading(true);
        try {
            const response = await invoke('get_logger', {});
            const data = response as Log[];
            console.log(data);

            // 追加する
            setLogger((prev) => [...prev, ...data]);
        } catch (error) {
            console.error(error);
        }
    };

    const clear = async () => {
        setLogger([]);
    };

    const onClose = () => {
        setOpen(false);
    };

    const onOpen = () => {
        fetchLogger();
        setOpen(true);
        setLoading(false);
    };

    const formatTime = (timestamp: string): string => {
        const date = new Date(timestamp);
        const hours = date.getHours().toString().padStart(2, '0');
        const minutes = date.getMinutes().toString().padStart(2, '0');
        const seconds = date.getSeconds().toString().padStart(2, '0');
        return `${hours}:${minutes}:${seconds}`;
    };

    return (
        <>
            <Button type="link" size="small" onClick={onOpen}>open log</Button>

            <Spin spinning={loading}>
                <Drawer title="History" onClose={onClose} open={open} closable={false}>
                    <ul>
                        {logger.map((item, index) => (
                            <li key={index}>{formatTime(item.timestamp)}|[{item.level}]: {item.message}</li>
                        ))}
                    </ul>

                    <Button type="primary" onClick={clear}>Clear</Button>
                </Drawer>

            </Spin>
        </>
    );
};

export { Logger };