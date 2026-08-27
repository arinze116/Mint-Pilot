use reqwest::Client;
use serde_json::json;

#[derive(Debug)]
pub struct TelegramNotifier {
    client: Client,
    bot_token: String,
    chat_id: String,
    enabled: bool,
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
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );
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
        collection: &str,
        wallet: &str,
        tx_hash: &str,
        quantity: u64,
        chain_id: u64,
    ) {
        let explorer = explorer_url(chain_id, tx_hash);
        self.send(&format!(
            "✅ <b>Mint Successful</b>\n\
             📦 Collection: <code>{collection}</code>\n\
             👛 Wallet: <code>{wallet}</code>\n\
             🔢 Quantity: {quantity}\n\
             🔗 Tx: <a href=\"{explorer}\">{tx_hash}</a>"
        ))
        .await;
    }

    pub async fn notify_failed(&self, collection: &str, wallet: &str, reason: &str) {
        self.send(&format!(
            "❌ <b>Mint Failed</b>\n\
             📦 Collection: <code>{collection}</code>\n\
             👛 Wallet: <code>{wallet}</code>\n\
             ⚠️ Reason: {reason}"
        ))
        .await;
    }

    pub async fn notify_skipped(&self, wallet: &str, reason: &str) {
        self.send(&format!(
            "⏭️ <b>Wallet Skipped</b>\n\
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
