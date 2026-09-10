use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
pub struct TelegramNotifier {
    client: Client,
    bot_token: String,
    chat_id: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct CollectionStats {
    total: StatBlock,
}

#[derive(Deserialize)]
struct StatBlock {
    floor_price: Option<f64>,
}

#[derive(Deserialize)]
struct CollectionInfo {
    #[allow(dead_code)]
    image_url: Option<String>,
    name: Option<String>,
}

impl TelegramNotifier {
    pub fn from_env() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
        let enabled = !bot_token.is_empty() && !chat_id.is_empty();
        Self {
            client: Client::new(),
            bot_token,
            chat_id,
            enabled,
        }
    }

    pub async fn send(&self, message: &str) {
        if !self.enabled {
            return;
        }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let _ = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": message,
                "parse_mode": "HTML"
            }))
            .send()
            .await;
    }

    async fn fetch_floor_price(&self, slug: &str) -> Option<f64> {
        let url = format!("https://api.opensea.io/api/v2/collections/{}/stats", slug);
        let resp = self
            .client
            .get(&url)
            .header(
                "x-api-key",
                std::env::var("OPENSEA_API_KEY").unwrap_or_default(),
            )
            .send()
            .await
            .ok()?;
        let stats: CollectionStats = resp.json().await.ok()?;
        stats.total.floor_price
    }

    async fn fetch_collection_name(&self, slug: &str) -> Option<String> {
        let url = format!("https://api.opensea.io/api/v2/collections/{}", slug);
        let resp = self
            .client
            .get(&url)
            .header(
                "x-api-key",
                std::env::var("OPENSEA_API_KEY").unwrap_or_default(),
            )
            .send()
            .await
            .ok()?;
        let info: CollectionInfo = resp.json().await.ok()?;
        info.name
    }

    pub async fn notify_scheduled(&self, collection: &str, phase: &str, starts_at: &str) {
        self.send(&format!(
            "🕐 <b>Mint Scheduled</b>\n\
             📦 Collection: <code>{collection}</code>\n\
             🎯 Phase: {phase}\n\
             ⏰ Starts at: {starts_at}"
        ))
        .await;
    }

    pub async fn notify_minted(
        &self,
        slug: &str,
        wallet: &str,
        tx_hash: &str,
        quantity: u64,
        mint_cost_eth: f64,
        chain_id: u64,
    ) {
        let explorer = explorer_url(chain_id, tx_hash);

        // Fetch floor price and collection name
        let floor = self.fetch_floor_price(slug).await.unwrap_or(0.0);
        let name = self
            .fetch_collection_name(slug)
            .await
            .unwrap_or_else(|| slug.to_string());

        // Calculate PnL
        let holding_value = floor * quantity as f64;
        let pnl = holding_value - mint_cost_eth;
        let pnl_pct = if mint_cost_eth > 0.0 {
            (pnl / mint_cost_eth) * 100.0
        } else {
            0.0
        };
        let pnl_sign = if pnl >= 0.0 { "+" } else { "" };
        let pnl_emoji = if pnl >= 0.0 { "📈" } else { "📉" };

        self.send(&format!(
            "✅ <b>Mint Successful</b>\n\
             ━━━━━━━━━━━━━━━━\n\
             🖼 <b>{name}</b>\n\
             👛 Wallet: <code>{wallet}</code>\n\
             ━━━━━━━━━━━━━━━━\n\
             🔢 Minted: {quantity}\n\
             💸 Mint Cost: {mint_cost_eth:.4} ETH\n\
             🏠 Floor Price: {floor:.4} ETH\n\
             💎 Holding Value: {holding_value:.4} ETH\n\
             ━━━━━━━━━━━━━━━━\n\
             {pnl_emoji} PnL: {pnl_sign}{pnl:.4} ETH ({pnl_sign}{pnl_pct:.1}%)\n\
             ━━━━━━━━━━━━━━━━\n\
             🔗 <a href=\"{explorer}\">View Transaction</a>"
        ))
        .await;
    }

    pub async fn notify_failed(&self, collection: &str, wallet: &str, reason: &str) {
        self.send(&format!(
            "❌ <b>Mint Failed</b>\n\
             ━━━━━━━━━━━━━━━━\n\
             📦 Collection: <code>{collection}</code>\n\
             👛 Wallet: <code>{wallet}</code>\n\
             ━━━━━━━━━━━━━━━━\n\
             ⚠️ Reason: {reason}"
        ))
        .await;
    }

    pub async fn notify_skipped(&self, wallet: &str, reason: &str) {
        self.send(&format!(
            "⏭️ <b>Wallet Skipped</b>\n\
             ━━━━━━━━━━━━━━━━\n\
             👛 Wallet: <code>{wallet}</code>\n\
             ⚠️ Reason: {reason}"
        ))
        .await;
    }
}

fn explorer_url(chain_id: u64, tx_hash: &str) -> String {
    match chain_id {
        1 => format!("https://etherscan.io/tx/{tx_hash}"),
        8453 => format!("https://basescan.org/tx/{tx_hash}"),
        763373 => format!("https://explorer.inkonchain.com/tx/{tx_hash}"),
        _ => format!("https://blockscan.com/tx/{tx_hash}"),
    }
}
