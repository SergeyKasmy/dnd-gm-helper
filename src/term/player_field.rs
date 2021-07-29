use crate::StatType;

#[derive(Copy, Clone)]
pub enum PlayerField {
    Name,
    Stat(StatType),
    SkillName(usize),
    SkillCD(usize),
}

impl PlayerField {
    pub fn next(&self) -> PlayerField {
        match self {
            PlayerField::Name => PlayerField::Stat(StatType::Strength),
            PlayerField::Stat(stat) => match stat {
                StatType::Strength => PlayerField::Stat(StatType::Dexterity),
                StatType::Dexterity => PlayerField::Stat(StatType::Poise),
                StatType::Poise => PlayerField::Stat(StatType::Wisdom),
                StatType::Wisdom => PlayerField::Stat(StatType::Intelligence),
                StatType::Intelligence => PlayerField::Stat(StatType::Charisma),
                StatType::Charisma => PlayerField::SkillName(0),
            },
            PlayerField::SkillName(i) => PlayerField::SkillCD(*i),
            PlayerField::SkillCD(i) => PlayerField::SkillName(*i + 1),
        }
    }

    pub fn prev(&self) -> PlayerField {
        match self {
            PlayerField::Name => PlayerField::Name,
            PlayerField::Stat(stat) => match stat {
                StatType::Strength => PlayerField::Name,
                StatType::Dexterity => PlayerField::Stat(StatType::Strength),
                StatType::Poise => PlayerField::Stat(StatType::Dexterity),
                StatType::Wisdom => PlayerField::Stat(StatType::Poise),
                StatType::Intelligence => PlayerField::Stat(StatType::Wisdom),
                StatType::Charisma => PlayerField::Stat(StatType::Intelligence),
            },
            PlayerField::SkillName(i) => {
                if *i == 0 {
                    PlayerField::Stat(StatType::Charisma)
                } else {
                    PlayerField::SkillCD(*i - 1)
                }
            }
            PlayerField::SkillCD(i) => PlayerField::SkillName(*i),
        }
    }
}
