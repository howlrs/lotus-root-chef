
export const SupportedBookSides = ['ask', 'bid'];
type BookSide = typeof SupportedBookSides[number];

export const SupportedOrderSides = ['buy', 'sell'];
type OrderSide = typeof SupportedOrderSides[number];

export const SupportedExchanges = ['bybit', 'bitbank', 'bitflyer'];
type ExchangeName = typeof SupportedExchanges[number];


export interface Controller {
    is_running: boolean;
    exchange: Exchange;
    board: Board;
    order: Order;
}

export interface Exchange {
    name: ExchangeName;
    key: string;
    secret: string;
    passphrase?: string;
    category?: string;
}

export interface Board {
    side: BookSide;

    hight: number;
    low: number;
    size: number;
}

export interface Order {
    symbol: string;

    side: OrderSide;
    size: number;
    is_post_only: boolean;

    tick_size: number;
    interval_sec: number;
}


export interface Ticker {
    symbol: string;
    ltp: number;
    volume24h: number;
    best_ask: number;
    best_bid: number;
}