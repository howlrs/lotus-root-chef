# レンコンシェフ/Lotus root Chef

※ [Pre], [ProtoType] since 2025/02/20

![summary_en](https://github.com/howlrs/lotus-root-chef/blob/images/images/summary_en.png?raw=true)
![summary_ja](https://github.com/howlrs/lotus-root-chef/blob/images/images/summary_ja.png?raw=true)

レンコンシェフは、指定価格帯の指定枚数の板に対して、追従しながら指値を行います。
デスクトップアプリが付随し、ローカルにて稼働します。指値速度はネットワーク(ping)に依存し、板探索はCPUに依存します。探索は価格板配列500でおおよそ10-50μs程度で、ping:20msかつAPI稼働通常時と仮定したとき取引所への板配置完了は100ms前後（取引所API稼働状態にも依ります）です。
以下パラメータで探索、注文の設定が可能です。

## Envs
- RUST_LOG: ログレベルの出力選択, select -> [trace, debug, info, warn, error], default -> debug
- OUTPUTLOGFILE: ログレベル[error]の出力ファイル先, select: any, default -> program_dir/output.log


## Supported Exchanges
- Bybit: 取引所API板取得最大:500の価格帯で対応（探索範囲は狭い）

## Planned support Exchanges
- Bitbank: 建玉取得がREST APIでリクエストリミットが限られていることに注意です。
- Bitflyer: 
- Okcoin Japan
- Bitget
- Binance Japan
- 

## Parameter
- Exchange
- Symbol
- Board
  - side
  - high price for range
  - low price for range
  - size
- Order
  - side
  - size
  - post_only
  - interval_sec


## Usage
デスクトップアプリを起動し、設定をSave後起動ボタンをクリックすると板取得及び探索、注文を開始します。
起動後は停止ボタンをクリックすると情報取得及び探索、注文を停止し、現在の設定の注文をキャンセルします。