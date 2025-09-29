# HyperTools

一个用于 Hyperliquid 交易所的命令行工具，支持查看账户余额、下单和取消订单等功能。

## 功能特性

- 查看账户余额
- 下单（支持现货和期货）
- 取消订单

## 安装

1. 确保已安装 Rust 和 Cargo
2. 克隆此仓库
3. 运行 `cargo build --release`

## 配置

在项目根目录创建 `.env` 文件，添加以下内容：

```
HYPERLIQUID_API_KEY=你的API密钥
HYPERLIQUID_API_SECRET=你的API密钥
```

## 使用方法

### 查看账户余额

```bash
hplq balance
```

### 查看账户仓位

```bash
hplq positions
```

### 下单

```bash
# 限价单
hplq order --asset BTC-USDT --market-type spot --side buy --order-type limit --price 50000 --quantity 0.1

# or in short
hplq order -a BTC-USDT -m sport -b -o limit -p 50000 -q 0.1

# 市价单
hplq order --asset BTC-USDT --market-type spot --side buy --order-type market --quantity 0.1

# or in short
hplq order -a BTC-USDT -m spot -b -o market -q 0.1
```

### 取消订单

```bash
hplq cancel --order-id 订单ID
```

## 参数说明

- `--symbol`: 交易对（例如：BTC-USDT）
- `--market-type`: 市场类型（spot/futures）
- `--side`: 订单方向（buy/sell）
- `--order-type`: 订单类型（limit/market）
- `--price`: 价格（限价单必填）
- `--quantity`: 数量
- `--order-id`: 订单ID（取消订单时使用）

## 注意事项

- 请确保在使用前正确配置 API 密钥
- 建议在测试网络上先进行测试
- 请妥善保管您的 API 密钥
