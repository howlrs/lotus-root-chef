import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

const resources = {
    en: {
        translation: {
            "running": {
                "label": "Running",
                "description": "Indicates whether the program is actively tracking limit orders."
            },
            "exchange": {
                "label": "Exchange",
                "description": "Specifies the exchange to connect to."
            },
            "key": {
                "label": "API Key",
                "description": "The API key provided by the exchange."
            },
            "secret": {
                "label": "API Secret",
                "description": "The API secret provided by the exchange."
            },
            "passphrase": {
                "label": "Passphrase",
                "description": "The passphrase for additional authentication."
            },
            "boardSide": {
                "label": "Order Book Side",
                "description": "Specifies the side of the order book (buy or sell) to monitor."
            },
            "boardHigh": {
                "label": "Order Book Upper Limit",
                "description": "Specifies the upper price limit for order book monitoring."
            },
            "boardLow": {
                "label": "Order Book Lower Limit",
                "description": "Specifies the lower price limit for order book monitoring."
            },
            "boardSize": {
                "label": "Order Book Depth",
                "description": "Specifies the number of levels in the order book to monitor."
            },
            "symbol": {
                "label": "Trading Symbol",
                "description": "Specifies the symbol used for trading."
            },
            "orderSide": {
                "label": "Order Side",
                "description": "Specifies the direction (buy or sell) of the limit order."
            },
            "orderSize": {
                "label": "Order Quantity",
                "description": "Specifies the quantity for placing the limit order."
            },
            "postOnly": {
                "label": "Post Only",
                "description": "Specifies a post-only order to ensure market maker execution."
            },
            "intervalSec": {
                "label": "Reorder Interval (Seconds)",
                "description": "Specifies the interval in seconds for executing new limit orders."
            },
            "button": {
                "send": {
                    "label": "Save",
                    "description": "Saves the configuration. When the settings are overwritten, monitoring will be stopped, orders will be cancelled, and monitoring will resume with the new settings."
                },
                "start": "Start",
                "stop": "Stop"
            }
        }
    },
    ja: {
        translation: {
            "running": {
                "label": "実行中",
                "description": "プログラムが指値注文追従を実行中かどうかを示します。"
            },
            "exchange": {
                "label": "取引所",
                "description": "接続する取引所を指定します。"
            },
            "key": {
                "label": "APIキー",
                "description": "取引所から提供されたAPIキーです。"
            },
            "secret": {
                "label": "APIシークレット",
                "description": "取引所から提供されたAPIシークレットです。"
            },
            "passphrase": {
                "label": "パスフレーズ",
                "description": "追加認証のためのパスフレーズです。"
            },
            "boardSide": {
                "label": "板のサイド",
                "description": "対象となる注文板のサイド（買いまたは売り）を指定します。"
            },
            "boardHigh": {
                "label": "板の高値",
                "description": "注文板の監視上限価格を指定します。"
            },
            "boardLow": {
                "label": "板の安値",
                "description": "注文板の監視下限価格を指定します。"
            },
            "boardSize": {
                "label": "板のサイズ",
                "description": "監視対象となる注文板の枚数を指定します。"
            },
            "symbol": {
                "label": "対象銘柄",
                "description": "取引する銘柄を指定します。"
            },
            "orderSide": {
                "label": "注文の方向",
                "description": "指値注文の方向（買いまたは売り）を指定します。"
            },
            "orderSize": {
                "label": "注文量",
                "description": "発注する指値注文の数量を指定します。"
            },
            "postOnly": {
                "label": "ポストオンリー",
                "description": "注文がマーケットメーカーとして機能するように、ポストオンリー注文を指定します。"
            },
            "intervalSec": {
                "label": "再注文間隔 (秒)",
                "description": "新たな指値注文を実行する間隔（秒）を指定します。"
            },
            "button": {
                "send": {
                    "label": "設定保存",
                    "description": "設定を保存します。設定が上書きされるとき、監視情報を停止・注文をキャンセルし、新しい設定で再度監視を処理を再開します。"
                },
                "start": "開始",
                "stop": "停止"
            }
        }
    }
};

i18n
    .use(initReactI18next)
    .init({
        resources,
        lng: "ja", // デフォルトの言語
        fallbackLng: "en",
        interpolation: {
            escapeValue: false // ReactはXSS対策済み
        }
    });

export default i18n;