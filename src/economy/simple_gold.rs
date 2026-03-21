const MAX_GOLD: u32 = 9_999_999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Player {
    pub gold: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaveData {
    pub gold: u32,
}

pub fn add_reward(player: &mut Player, amount: u32) {
    player.gold = player.gold.saturating_add(amount).min(MAX_GOLD);
}

pub fn purchase_item(player: &mut Player, cost: u32) -> bool {
    if player.gold < cost {
        return false;
    }

    player.gold -= cost;
    true
}

pub fn serialize_save(player: &Player) -> SaveData {
    SaveData {
        gold: player.gold.min(MAX_GOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_reward_increases_gold() {
        let mut player = Player { gold: 250 };

        add_reward(&mut player, 50);

        assert_eq!(player.gold, 300);
    }

    #[test]
    fn add_reward_respects_cap() {
        let mut player = Player { gold: 9_999_990 };

        add_reward(&mut player, 25);

        assert_eq!(player.gold, MAX_GOLD);
    }

    #[test]
    fn purchase_item_succeeds_when_affordable() {
        let mut player = Player { gold: 500 };

        let purchased = purchase_item(&mut player, 125);

        assert!(purchased);
        assert_eq!(player.gold, 375);
    }

    #[test]
    fn purchase_item_fails_when_not_affordable() {
        let mut player = Player { gold: 40 };

        let purchased = purchase_item(&mut player, 100);

        assert!(!purchased);
        assert_eq!(player.gold, 40);
    }

    #[test]
    fn serialize_save_carries_gold() {
        let player = Player { gold: 12_345 };

        let save = serialize_save(&player);

        assert_eq!(save.gold, 12_345);
    }

    #[test]
    fn serialize_save_clamps_invalid_gold() {
        let player = Player { gold: u32::MAX };

        let save = serialize_save(&player);

        assert_eq!(save.gold, MAX_GOLD);
    }
}
