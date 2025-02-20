use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct ApiCancelResponse {
    result: Option<Value>,
    #[serde(rename = "retCode")]
    ret_code: i64,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    #[serde(rename = "retExtInfo")]
    pub ret_ext_info: HashMap<String, Value>,
    time: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiOrderResponse {
    #[serde(rename = "retCode")]
    pub ret_code: i16,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: OrderStatus,
    #[serde(rename = "retExtInfo")]
    pub ret_ext_info: HashMap<String, Value>,
    pub time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatus {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "orderLinkId")]
    pub order_link_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDefaultResponse {
    #[serde(rename = "retCode")]
    pub ret_code: i64,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: ApiResult,
    #[serde(rename = "retExtInfo")]
    pub ret_ext_info: HashMap<String, Value>,
    pub time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResult {
    pub category: String,
    pub list: Value,
    // #[serde(rename = "nextPageCursor")]
    // pub next_page_cursor: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub symbol: String,
    #[serde(rename = "contractType")]
    pub contract_type: String,
    pub status: String,
    #[serde(rename = "baseCoin")]
    pub base_coin: String,
    #[serde(rename = "quoteCoin")]
    pub quote_coin: String,
    #[serde(rename = "launchTime")]
    pub launch_time: String,
    #[serde(rename = "deliveryTime")]
    pub delivery_time: String,
    #[serde(rename = "deliveryFeeRate")]
    pub delivery_fee_rate: String,
    #[serde(rename = "priceScale")]
    pub price_scale: String,
    #[serde(rename = "leverageFilter")]
    pub leverage_filter: LeverageFilter,
    #[serde(rename = "priceFilter")]
    pub price_filter: PriceFilter,
    #[serde(rename = "lotSizeFilter")]
    pub lot_size_filter: LotSizeFilter,
    #[serde(rename = "unifiedMarginTrade")]
    pub unified_margin_trade: bool,
    #[serde(rename = "fundingInterval")]
    pub funding_interval: i64,
    #[serde(rename = "settleCoin")]
    pub settle_coin: String,
    #[serde(rename = "copyTrading")]
    pub copy_trading: String,
    #[serde(rename = "upperFundingRate")]
    pub upper_funding_rate: String,
    #[serde(rename = "lowerFundingRate")]
    pub lower_funding_rate: String,
    #[serde(rename = "isPreListing")]
    pub is_pre_listing: bool,
    #[serde(rename = "preListingInfo")]
    pub pre_listing_info: Option<PreListingInfo>,
    #[serde(rename = "riskParameters")]
    pub risk_parameters: RiskParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeverageFilter {
    #[serde(rename = "minLeverage")]
    pub min_leverage: String,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: String,
    #[serde(rename = "leverageStep")]
    pub leverage_step: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceFilter {
    #[serde(rename = "minPrice")]
    pub min_price: String,
    #[serde(rename = "maxPrice")]
    pub max_price: String,
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotSizeFilter {
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: String,
    #[serde(rename = "minOrderQty")]
    pub min_order_qty: String,
    #[serde(rename = "qtyStep")]
    pub qty_step: String,
    #[serde(rename = "postOnlyMaxOrderQty")]
    pub post_only_max_order_qty: String,
    #[serde(rename = "maxMktOrderQty")]
    pub max_mkt_order_qty: String,
    #[serde(rename = "minNotionalValue")]
    pub min_notional_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreListingInfo {
    #[serde(rename = "curAuctionPhase")]
    pub cur_auction_phase: String,
    pub phases: Vec<Phase>,
    #[serde(rename = "auctionFeeInfo")]
    pub auction_fee_info: AuctionFeeInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phase {
    pub phase: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionFeeInfo {
    #[serde(rename = "auctionFeeRate")]
    pub auction_fee_rate: String,
    #[serde(rename = "takerFeeRate")]
    pub taker_fee_rate: String,
    #[serde(rename = "makerFeeRate")]
    pub maker_fee_rate: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskParameters {
    #[serde(rename = "priceLimitRatioX")]
    pub price_limit_ratio_x: String,
    #[serde(rename = "priceLimitRatioY")]
    pub price_limit_ratio_y: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerInfo {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "indexPrice")]
    pub index_price: String,
    #[serde(rename = "markPrice")]
    pub mark_price: String,
    #[serde(rename = "prevPrice24h")]
    pub prev_price_24h: String,
    #[serde(rename = "price24hPcnt")]
    pub price_24h_pcnt: String,
    #[serde(rename = "highPrice24h")]
    pub high_price_24h: String,
    #[serde(rename = "lowPrice24h")]
    pub low_price_24h: String,
    #[serde(rename = "prevPrice1h")]
    pub prev_price_1h: String,
    #[serde(rename = "openInterest")]
    pub open_interest: String,
    #[serde(rename = "openInterestValue")]
    pub open_interest_value: String,
    #[serde(rename = "turnover24h")]
    pub turnover_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
    #[serde(rename = "fundingRate")]
    pub funding_rate: String,
    #[serde(rename = "nextFundingTime")]
    pub next_funding_time: String,
    #[serde(rename = "predictedDeliveryPrice")]
    pub predicted_delivery_price: String,
    #[serde(rename = "basisRate")]
    pub basis_rate: String,
    #[serde(rename = "deliveryFeeRate")]
    pub delivery_fee_rate: String,
    #[serde(rename = "deliveryTime")]
    pub delivery_time: String,
    #[serde(rename = "ask1Size")]
    pub ask1_size: String,
    #[serde(rename = "bid1Price")]
    pub bid1_price: String,
    #[serde(rename = "ask1Price")]
    pub ask1_price: String,
    #[serde(rename = "bid1Size")]
    pub bid1_size: String,
    pub basis: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiOrderbook {
    pub s: String,           // Symbol
    pub b: Vec<[String; 2]>, // Bids [price, size]
    pub a: Vec<[String; 2]>, // Asks [price, size]
    pub u: i64,              // Update ID
    pub seq: i64,            // Sequence number
}
