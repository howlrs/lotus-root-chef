import { Form, Input, Button, InputNumber, Select, FormProps, Switch, message, Spin, FloatButton, Flex } from "antd";
import { Board, Controller, Exchange, Order, SupportedExchanges, SupportedBookSides, SupportedOrderSides, Ticker } from "./models";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { startController, stopController } from "./crud-controller";

import { CaretRightOutlined, PauseOutlined } from '@ant-design/icons';

import { History } from "./history";
import { Logger } from "./logger";

// 言語分岐
import { useTranslation } from 'react-i18next';
import './i18n';


interface Instrument {
    symbol: string;
    ltp: number;
    volume24h: number;
    price_tick: number;
    size_tick: number;
    size_min: number;
}

const defaultExchange: Exchange = {
    name: '',
    key: '',
    secret: '',
    passphrase: ''
};

const defaultBoard: Board = {
    side: SupportedBookSides[0],
    hight: 15_000_000,
    low: 15_000_000 * 0.9,
    size: 100
};

const defaultOrder: Order = {
    symbol: '',
    size: 100,
    side: SupportedOrderSides[1],
    is_post_only: true,
    tick_size: 0,
    interval_sec: 5
};

const defaultController: Controller = {
    is_running: false,
    exchange: defaultExchange,
    board: defaultBoard,
    order: defaultOrder
};

export const FormComponent = () => {
    const { t, i18n } = useTranslation();
    const [language, setLanguage] = useState('ja');

    const [controller, setController] = useState<Controller>(defaultController);
    const [form] = Form.useForm<Controller>();
    const [instruments, setInstruments] = useState<Instrument[]>([]);
    const [selectInstrument, setSelectInstrument] = useState<Instrument | null>(null);

    // 取引所非対応要素の表示設定
    const [showPassphrase, setShowPassphrase] = useState(false);
    useEffect(() => {
        switch (controller.exchange.name) {
            case 'bitbank':
                setShowPassphrase(true);
                break;
            default:
                setShowPassphrase(false);
                break;
        }
    }, [controller.exchange.name]);


    const [history, setHistory] = useState<Controller[]>([]);

    const [loading, setLoading] = useState(false);

    const changeLanguage = (lang: string) => {
        setLanguage(lang);
        i18n.changeLanguage(lang);
    };

    let supportedExchanges = SupportedExchanges.map((exchange) => {
        return { label: exchange, value: exchange };
    });
    let supportedBookSides = SupportedBookSides.map((side: any) => {
        return { label: side, value: side };
    });
    let supportedOrderSides = SupportedOrderSides.map((side: any) => {
        return { label: side, value: side };
    });

    const start = async () => {
        setLoading(true);
        try {
            const controller = await startController();
            setController(controller);
            setHistory([...history, controller]);
            form.setFieldValue('is_running', controller.is_running);
            message.open({
                type: 'success',
                duration: 3,
                content: `Board tracking is ${controller.is_running ? 'is running' : 'was stoped'}`
            });
        } catch (error: any) {
            console.error(error);
            // error message
            message.open({
                type: 'error',
                duration: 3,
                content: `${error.msg}, cause: ${error.cause}`
            });
        } finally {
            setLoading(false);
        }
    };

    const stop = async () => {
        setLoading(true);
        try {
            const controller = await stopController();
            setController(controller);
            form.setFieldValue('is_running', controller.is_running);
            message.open({
                type: 'success',
                duration: 3,
                content: `Board tracking ${controller.is_running ? 'is running' : 'was stoped'}`
            });
        } catch (error: any) {
            console.error(error);
            // error message
            message.open({
                type: 'error',
                duration: 3,
                content: `${error.msg}, cause: ${error.cause}`
            });
        } finally {
            setLoading(false);
        }
    };


    const onFinish: FormProps<Controller>['onFinish'] = async (values: Controller) => {
        setLoading(true);
        try {
            values.order.tick_size = selectInstrument?.price_tick || 0;

            const res = await invoke('post_controller', { value: values });
            console.log(res);
            message.open({
                type: 'success',
                duration: 3,
                content: 'Setting updated successfully'
            });
        } catch (error: any) {
            console.error(error);
            message.open({
                type: 'error',
                duration: 3,
                content: `${error.msg}, cause: ${error.cause}`
            });
        } finally {
            setLoading(false);
        }
    };

    const fetchInstruments = async (exchange_name: string) => {
        try {
            const res = await invoke('get_instruments', { exchange_name: exchange_name });
            setInstruments(res as Instrument[]);
        } catch (error) {
            console.error(error);
        }
    }

    const fetchTicker = async (symbol: string) => {
        try {
            const exchange_name = controller.exchange.name;
            if (!exchange_name) {
                throw new Error('exchange name is empty');
            }
            if (!symbol) {
                throw new Error('symbol is empty');
            }

            const res = await invoke('get_ticker', { exchange_name: exchange_name, symbol: symbol });
            const ticker = res as Ticker;
            setController((prev) => {
                const updated = {
                    ...prev,
                    board: {
                        ...prev.board,
                        hight: ticker.best_ask,
                        low: ticker.best_bid * 0.99,
                        size: ticker.volume24h / 24
                    }
                };
                form.setFieldsValue(updated);
                return updated;
            });




            console.log(res);
        } catch (error) {
            console.error(error);
        }
    }

    const onChangeForm = (_: any, values: Controller) => {
        console.log(values);

        setController(values);
    }

    // 履歴選択時の処理
    // 現在の設定項目へ代入する
    const onSelectController = (controller: Controller) => {
        console.log('on select: ');
        console.log(controller);
        setController(controller);
        form.setFieldsValue(controller);
    };


    return (
        <>
            <Spin spinning={loading}>
                {
                    controller.is_running ? <FloatButton icon={<PauseOutlined />} onClick={stop} /> : <FloatButton icon={<CaretRightOutlined />} onClick={start} />
                }

                {/* 言語切替 */}
                {/* 上部左端固定 */}
                <Flex justify="space-around" style={{ position: 'fixed', top: 0, left: 0, width: '100vw' }}>
                    <Button size="small" type="link" onClick={
                        // toggle language
                        () => changeLanguage(language === 'en' ? 'ja' : 'en')
                    }
                    >
                        {language === 'en' ? '日本語へ言語変更' : 'Switch to English'}
                    </Button>
                    <Logger />
                    <History controllers={history} onSelectController={onSelectController} />
                </Flex>

                <Form
                    name="basic"
                    form={form}
                    labelCol={{ span: 8 }}
                    wrapperCol={{ span: 16 }}
                    initialValues={controller}
                    style={{ maxWidth: 640, textAlign: 'left' }}
                    onFinish={onFinish}
                    autoComplete="off"
                    onValuesChange={onChangeForm}
                >

                    <Form.Item<Controller>
                        label={t('running.label')}
                        tooltip={t('running.description')}
                        name={"is_running"}
                    >
                        <Switch disabled />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('exchange.label')}
                        tooltip={t('exchange.description')}
                        name={["exchange", "name"]}
                        rules={[{ required: true, message: 'Please input exchange name' }]}
                    >
                        <Select options={supportedExchanges} onChange={(v) => fetchInstruments(v)} />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('key.label')}
                        tooltip={t('key.description')}
                        name={["exchange", "key"]}
                        rules={[{ required: true, message: 'Please input exchange key' }]}
                    >
                        <Input />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('secret.label')}
                        tooltip={t('secret.description')}
                        name={["exchange", "secret"]}
                        rules={[{ required: true, message: 'Please input your exchange secret key' }]}
                    >
                        <Input.Password />
                    </Form.Item>

                    {showPassphrase ? (
                        <Form.Item<Controller>
                            label={t('passphrase.label')}
                            tooltip={t('passphrase.description')}
                            name={["exchange", "passphrase"]}
                            rules={[{ required: false, message: 'Please input exchange passphrase' }]}
                        >
                            <Input />
                        </Form.Item>
                    ) :
                        null
                    }

                    <Form.Item<Controller>
                        label={t('boardSide.label')}
                        tooltip={t('boardSide.description')}
                        name={["board", "side"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <Select options={supportedBookSides} onChange={
                            async (v) => {
                                console.log(v);

                                switch (v) {
                                    case 'bid':
                                        setController((prev) => {
                                            return {
                                                ...prev,
                                                order: {
                                                    ...prev.order,
                                                    side: 'buy'
                                                }
                                            };
                                        });
                                        form.setFieldValue(['order', 'side'], 'buy');
                                        break;
                                    case 'ask':
                                        setController((prev) => {
                                            return {
                                                ...prev,
                                                order: {
                                                    ...prev.order,
                                                    side: 'sell'
                                                }
                                            };
                                        });
                                        form.setFieldValue(['order', 'side'], 'sell');
                                        break;
                                }
                            }
                        } />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('boardHigh.label')}
                        tooltip={t('boardHigh.description')}
                        name={["board", "hight"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <InputNumber />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('boardLow.label')}
                        tooltip={t('boardLow.description')}
                        name={["board", "low"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <InputNumber />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('boardSize.label')}
                        tooltip={t('boardSize.description')}
                        name={["board", "size"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <InputNumber />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('symbol.label')}
                        tooltip={t('symbol.description')}
                        name={["order", "symbol"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <Select options={instruments.map((instrument) => {
                            return { label: instrument.symbol, value: instrument.symbol }
                        })} optionFilterProp="label" showSearch onChange={
                            async (v) => {
                                const instrument = instruments.find((instrument) => instrument.symbol === v);
                                setSelectInstrument(instrument || null);

                                await fetchTicker(v);
                            }
                        } />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('orderSide.label')}
                        tooltip={t('orderSide.description')}
                        name={["order", "side"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <Select options={supportedOrderSides} onChange={
                            async (v) => {
                                console.log(v);

                                switch (v) {
                                    case 'buy':
                                        setController((prev) => {
                                            return {
                                                ...prev,
                                                board: {
                                                    ...prev.board,
                                                    side: 'bid'
                                                }
                                            };
                                        });
                                        form.setFieldValue(['board', 'side'], 'bid');
                                        break;
                                    case 'sell':
                                        setController((prev) => {
                                            return {
                                                ...prev,
                                                board: {
                                                    ...prev.board,
                                                    side: 'ask'
                                                }
                                            };
                                        });
                                        form.setFieldValue(['board', 'side'], 'ask');
                                        break;
                                }
                            }
                        } />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('orderSize.label')}
                        tooltip={t('orderSize.description')}
                        name={["order", "size"]}

                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <InputNumber />
                    </Form.Item>

                    <Form.Item<Controller>
                        label={t('postOnly.label')}
                        tooltip={t('postOnly.description')}
                        name={["order", "is_post_only"]}
                    >
                        <Switch />
                    </Form.Item>


                    <Form.Item<Controller>
                        label={t('intervalSec.label')}
                        tooltip={t('intervalSec.description')}
                        name={["order", "interval_sec"]}
                        rules={[{ required: true, message: 'Please input your setting' }]}
                    >
                        <InputNumber />
                    </Form.Item>


                    <Form.Item label={t('button.send.label')} tooltip={t('button.send.description')}>
                        <Button type="primary" htmlType="submit">
                            {t('button.send.label')}
                        </Button>
                    </Form.Item>
                </Form >
            </Spin>
        </>
    );
};
