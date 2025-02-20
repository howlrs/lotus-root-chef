import { useEffect, useState } from "react";
import { Controller } from "./models";
import { Button, Drawer } from "antd";

// Controllerインターフェースにkeyプロパティを追加した型を定義
type ControllerWithKey = Controller & { key: string };

interface HistoryProps {
    controllers: Controller[];
    onSelectController: (controller: Controller) => void;
}

const History = ({ controllers, onSelectController }: HistoryProps) => {
    const [open, setOpen] = useState(false);
    const [history, setHistory] = useState<ControllerWithKey[]>([]);

    const addKey = (controller: Controller): ControllerWithKey => {
        return { ...controller, key: `${controller.exchange.name}:${controller.order.symbol}:${controller.order.side}` };
    }

    const updateHistory = (controllers: Controller[]) => {
        setHistory(controllers.map(addKey));
    };


    useEffect(() => {
        // update history when controllers are updated
        updateHistory(controllers);
    }, [controllers]);


    const handleItemClick = (item: ControllerWithKey) => {
        onSelectController(item); // クリックされたアイテムのController情報を親コンポーネントに渡す
    };

    const onClose = () => {
        setOpen(false);
    };

    const onOpen = () => {
        setOpen(true);
    };

    if (history.length === 0) {
        return null;
    }
    return (
        <>
            <Button type="link" size="small" onClick={onOpen}>open history</Button>
            <Drawer title="History" onClose={onClose} open={open} closable={false}>
                <ul>
                    {history.map((item, index) => (
                        <li key={index}>
                            <Button type="text" key={item.key} onClick={() => handleItemClick(item)}>{item.key}</Button>
                        </li>
                    ))}
                </ul>
            </Drawer>
        </>
    );
};

export { History };