// ============================================================
// YODPAY – Offline-First Payment Core (C++)
// ============================================================

#include <iostream>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>
#include <chrono>
#include <iomanip>
#include <sstream>
#include <stdexcept>

enum class WalletTier { Standard, Business, Enterprise };
enum class TxStatus   { PendingOffline, ConfirmedLocal, Settled };
enum class NetworkMode { Normal, Partitioned, Extreme };

std::string tier_to_string(WalletTier t) {
    switch (t) {
        case WalletTier::Standard:   return "Standard";
        case WalletTier::Business:   return "Business";
        case WalletTier::Enterprise: return "Enterprise";
    }
    return "Unknown";
}

std::string status_to_string(TxStatus s) {
    switch (s) {
        case TxStatus::PendingOffline:  return "PendingOffline";
        case TxStatus::ConfirmedLocal:  return "ConfirmedLocal";
        case TxStatus::Settled:         return "Settled";
    }
    return "Unknown";
}

std::string mode_to_string(NetworkMode m) {
    switch (m) {
        case NetworkMode::Normal:      return "Normal (3f+1)";
        case NetworkMode::Partitioned: return "Partitioned (2f+1)";
        case NetworkMode::Extreme:     return "Extreme (1f+1)";
    }
    return "Unknown";
}

struct Wallet {
    std::string id;
    uint64_t balance = 0;
    uint64_t offline_limit = 0;
    uint64_t spent_offline = 0;
    uint64_t nonce = 0;
    WalletTier tier = WalletTier::Standard;

    static uint64_t default_limit(WalletTier t) {
        switch (t) {
            case WalletTier::Standard:   return 1500;
            case WalletTier::Business:   return 12000;
            case WalletTier::Enterprise: return 60000;
        }
        return 1500;
    }
};

struct Transaction {
    std::string id;
    std::string from;
    std::string to;
    uint64_t amount = 0;
    uint64_t nonce = 0;
    uint64_t timestamp = 0;
    TxStatus status = TxStatus::PendingOffline;
    std::vector<std::string> confirmed_by;
};

class YodPayNode {
public:
    std::string node_id;
    std::unordered_map<std::string, Wallet> wallets;
    std::unordered_map<std::string, Transaction> pending;
    std::unordered_map<std::string, Transaction> confirmed;
    std::unordered_map<std::string, Transaction> settled;
    std::unordered_set<std::string> spent_nonces;
    NetworkMode mode = NetworkMode::Normal;
    size_t known_peers = 7;

    explicit YodPayNode(const std::string& id) : node_id(id) {}

    size_t required_confirmations() const {
        switch (mode) {
            case NetworkMode::Normal: {
                size_t f = known_peers / 3;
                return 2 * f + 1;
            }
            case NetworkMode::Partitioned: {
                size_t f = known_peers / 2;
                return f + 1;
            }
            case NetworkMode::Extreme:
                return 1;
        }
        return 1;
    }

    void create_wallet(const std::string& id, uint64_t balance, WalletTier tier) {
        Wallet w;
        w.id = id;
        w.balance = balance;
        w.tier = tier;
        w.offline_limit = Wallet::default_limit(tier);
        w.spent_offline = 0;
        w.nonce = 0;

        wallets[id] = w;
        std::cout << "[YODPAY] Wallet '" << id << "' created"
                  << " | Balance: " << balance
                  << " | Offline limit: " << w.offline_limit
                  << " | Tier: " << tier_to_string(tier) << "\n";
    }

    std::string send_offline(const std::string& from, const std::string& to, uint64_t amount) {
        auto it = wallets.find(from);
        if (it == wallets.end()) {
            throw std::runtime_error("Wallet not found: " + from);
        }
        Wallet& w = it->second;

        if (w.spent_offline + amount > w.offline_limit) {
            throw std::runtime_error(
                "Offline limit exceeded. Spent: " + std::to_string(w.spent_offline) +
                ", Limit: " + std::to_string(w.offline_limit) +
                ", Tried: " + std::to_string(amount)
            );
        }

        if (amount > w.balance) {
            throw std::runtime_error("Insufficient balance");
        }

        std::string nonce_key = from + ":" + std::to_string(w.nonce);
        if (spent_nonces.count(nonce_key)) {
            throw std::runtime_error("Nonce already used – possible double-spend");
        }

        auto now = std::chrono::duration_cast<std::chrono::seconds>(
            std::chrono::system_clock::now().time_since_epoch()).count();

        Transaction tx;
        tx.id = "tx_" + from + "_" + std::to_string(w.nonce) + "_" + std::to_string(now);
        tx.from = from;
        tx.to = to;
        tx.amount = amount;
        tx.nonce = w.nonce;
        tx.timestamp = now;
        tx.status = TxStatus::PendingOffline;

        spent_nonces.insert(nonce_key);
        w.nonce++;
        w.spent_offline += amount;
        w.balance -= amount;

        pending[tx.id] = tx;

        std::cout << "[YODPAY] Offline payment created: " << from << " → " << to
                  << " | Amount: " << amount << " | Tx: " << tx.id << "\n";
        return tx.id;
    }

    void confirm_local(const std::string& tx_id, const std::string& confirmer) {
        auto it = pending.find(tx_id);
        if (it == pending.end()) {
            throw std::runtime_error("Transaction not found: " + tx_id);
        }

        Transaction& tx = it->second;

        bool already = false;
        for (const auto& c : tx.confirmed_by) {
            if (c == confirmer) { already = true; break; }
        }
        if (!already) {
            tx.confirmed_by.push_back(confirmer);
        }

        size_t required = required_confirmations();

        if (tx.confirmed_by.size() >= required) {
            tx.status = TxStatus::ConfirmedLocal;
            confirmed[tx_id] = tx;
            pending.erase(it);
            std::cout << "[YODPAY] Tx " << tx_id << " reached LOCAL FINALITY"
                      << " (mode: " << mode_to_string(mode)
                      << ", confirmations: " << required << ")\n";
        } else {
            std::cout << "[YODPAY] Tx " << tx_id << " has "
                      << tx.confirmed_by.size() << "/" << required << " confirmations\n";
        }
    }

    void settle(const std::string& tx_id) {
        auto it = confirmed.find(tx_id);
        if (it == confirmed.end()) {
            throw std::runtime_error("Transaction not found in confirmed: " + tx_id);
        }

        Transaction tx = it->second;
        tx.status = TxStatus::Settled;
        settled[tx_id] = tx;
        confirmed.erase(it);

        std::cout << "[YODPAY] Tx " << tx_id << " SETTLED on the wider network\n";
    }

    void set_mode(NetworkMode m) {
        mode = m;
        std::cout << "[YODPAY] Network mode changed to: " << mode_to_string(m) << "\n";
    }

    void print_status() const {
        std::cout << "\n========== YODPAY NODE STATUS ==========\n";
        std::cout << "Node: " << node_id << " | Mode: " << mode_to_string(mode) << "\n";
        std::cout << "----------------------------------------\n";
        for (const auto& [id, w] : wallets) {
            std::cout << "Wallet " << std::left << std::setw(10) << id
                      << " | Balance: " << std::setw(6) << w.balance
                      << " | Offline: " << w.spent_offline << "/" << w.offline_limit
                      << " | Nonce: " << w.nonce << "\n";
        }
        std::cout << "----------------------------------------\n";
        std::cout << "Pending:   " << pending.size() << "\n";
        std::cout << "Confirmed: " << confirmed.size() << "\n";
        std::cout << "Settled:   " << settled.size() << "\n";
        std::cout << "========================================\n\n";
    }
};

int main() {
    try {
        YodPayNode node("node-01");

        node.create_wallet("alice", 50000, WalletTier::Business);
        node.create_wallet("bob", 20000, WalletTier::Standard);
        node.create_wallet("clinic", 100000, WalletTier::Enterprise);

        std::cout << "\n--- Normal offline payment ---\n";
        std::string tx1 = node.send_offline("alice", "bob", 8000);

        node.confirm_local(tx1, "node-01");
        node.confirm_local(tx1, "node-02");
        node.confirm_local(tx1, "node-03");

        std::cout << "\n--- Trying to exceed offline limit ---\n";
        try {
            node.send_offline("alice", "bob", 50000);
        } catch (const std::exception& e) {
            std::cout << "Correctly blocked: " << e.what() << "\n";
        }

        std::cout << "\n--- Switching to Extreme isolation mode (1f+1) ---\n";
        node.set_mode(NetworkMode::Extreme);
        std::string tx2 = node.send_offline("clinic", "alice", 15000);
        node.confirm_local(tx2, "node-01");

        std::cout << "\n--- Settling when network returns ---\n";
        node.settle(tx1);

        node.print_status();

        std::cout << "Core offline payment protocol is complete.\n";

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}
