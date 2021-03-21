use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
enum Stat {
    MainStat,                         // 주스탯
    MainStatPercent,                  // 주스탯퍼
    MainStatPercentExempt,            // 스탯퍼 적용되지 않는 주스탯
    SubStat,                          // 부스탯
    SubStatPercent,                   // 부스탯퍼
    SubStatPercentExempt,             // 스탯퍼 적용되지 않는 부스탯
    Atk,                              // 공마
    AtkPercent,                       // 공마퍼
    DmgPercent,                       // 뎀퍼
    BossDmgPercent,                   // 보공
    FinalDmgPercent,                  // 최종뎀
    IgnoreGuardPercent,               // 방무
    IgnoreElementalResistancePercent, // 속성내성무시
    CritRatePercent,                  // 크확
    CritDmgPercent,                   // 크뎀
    WeaponConstant,                   // 무기상수
    ClassConstant,                    // 직업상수
    MasteryPercent,                   // 숙련도

    // 추가스탯; Y136:Y141
    ExtraDmgPercent,         // 추가뎀퍼
    ExtraBossDmgPercent,     // 추가보공
    ExtraFinalDmgPercent,    // 추가최종뎀
    ExtraIgnoreGuardPercent, // 추가방무
    ExtraCritRatePercent,    // 추가크확
    ExtraCritDmgPercent,     // 추가크뎀
}
/// Members that end with `percent` are measured in units of %.  
/// ex) dmg_percent = 50 == DMG% = 50%  
/// Listed in X121:Y135 of Spec Simulator 0.9.4  
/// 메용 포함  
#[derive(Clone)]
struct Stats {
    inner: HashMap<Stat, f64>,
}

impl Stats {
    fn init() -> Self {
        let mut inner = HashMap::new();

        inner.insert(Stat::MainStat, 4395 as f64);
        inner.insert(Stat::MainStatPercent, 331 as f64);
        inner.insert(Stat::MainStatPercentExempt, 13780 as f64);
        inner.insert(Stat::SubStat, 2014 as f64);
        inner.insert(Stat::SubStatPercent, 103 as f64);
        inner.insert(Stat::SubStatPercentExempt, 440 as f64);
        inner.insert(Stat::Atk, 1954 as f64);
        inner.insert(Stat::AtkPercent, 97 as f64);
        inner.insert(Stat::DmgPercent, 57 as f64);
        inner.insert(Stat::BossDmgPercent, 236 as f64);
        inner.insert(Stat::FinalDmgPercent, 30 as f64);
        inner.insert(Stat::IgnoreGuardPercent, 78.73660272);
        inner.insert(Stat::IgnoreElementalResistancePercent, 5 as f64);
        inner.insert(Stat::CritRatePercent, 70 as f64);
        inner.insert(Stat::CritDmgPercent, 65 as f64);
        inner.insert(Stat::WeaponConstant, 1.2);
        inner.insert(Stat::ClassConstant, 1.0);
        inner.insert(Stat::MasteryPercent, 90 as f64);

        inner.insert(Stat::ExtraDmgPercent, 3.2);
        inner.insert(Stat::ExtraBossDmgPercent, 0 as f64);
        inner.insert(Stat::ExtraFinalDmgPercent, 55.94);
        inner.insert(Stat::ExtraIgnoreGuardPercent, 42.88);
        inner.insert(Stat::ExtraCritRatePercent, 0 as f64);
        inner.insert(Stat::ExtraCritDmgPercent, 0 as f64);

        Self { inner }
    }

    fn get(&self, stat: Stat) -> f64 {
        *self.inner.get(&stat).unwrap()
    }

    fn display_stat_atk(&self) -> f64 {
        let main_stat_final = (self.get(Stat::MainStat)
            * (1.0 + self.get(Stat::MainStatPercent) / 100.0)
            + self.get(Stat::MainStatPercentExempt))
        .floor();
        let sub_stat_final = (self.get(Stat::SubStat)
            * (1.0 + self.get(Stat::SubStatPercent) / 100.0)
            + self.get(Stat::SubStatPercentExempt))
        .floor();
        let atk_final = (self.get(Stat::Atk) * (1.0 + self.get(Stat::AtkPercent) / 100.0)).floor();

        let stat_atk = (main_stat_final * 4.0 + sub_stat_final) * 0.01;
        let stat_atk = stat_atk * atk_final;
        let stat_atk = stat_atk * self.get(Stat::WeaponConstant) * self.get(Stat::ClassConstant);
        let stat_atk = stat_atk * (1.0 + self.get(Stat::DmgPercent) / 100.0);
        let stat_atk = stat_atk * (1.0 + self.get(Stat::FinalDmgPercent) / 100.0);

        stat_atk.floor()
    }

    fn get_stats_with_changes(&self, link: &LinkSkill) -> Self {
        let mut new_stats = self.clone();
        for effect in &link.effects {
            let stat = &effect.stat;
            let f = new_stats.inner.get_mut(&stat).unwrap();
            match stat {
                Stat::CritRatePercent | Stat::ExtraCritRatePercent => {
                    *f += effect.amount;
                    if *f > 100.0 {
                        *f = 100.0;
                    }
                }
                Stat::IgnoreGuardPercent | Stat::ExtraIgnoreGuardPercent => {
                    *f = add_ignore_guard_percents(vec![*f, effect.amount]);
                }
                _ => *f += effect.amount,
            }
        }

        new_stats
    }
}

struct CalculatorInfo {
    target_boss_guard_percent: usize,                // 보스 방어율
    target_boss_elemental_resistance_percent: usize, // 보스 속성내성

    free_link_skill_spaces: usize,      // 링크스킬 칸수
    used_hyper_stat_levels: Vec<usize>, // 잡하이퍼스탯 찍은 레벨들
    level: usize,                       // 레벨
}

impl CalculatorInfo {
    fn init() -> Self {
        Self {
            target_boss_guard_percent: 300,
            target_boss_elemental_resistance_percent: 50,
            free_link_skill_spaces: 10,
            used_hyper_stat_levels: vec![10],
            level: 251,
        }
    }

    /// 보스상대 한줄뎀  
    /// 렙차, 아케인포스 무시
    fn boss_line_dmg(&self, stats: Stats) -> f64 {
        let main_stat_final = (stats.get(Stat::MainStat)
            * (1.0 + stats.get(Stat::MainStatPercent) / 100.0)
            + stats.get(Stat::MainStatPercentExempt))
        .floor();
        let sub_stat_final = (stats.get(Stat::SubStat)
            * (1.0 + stats.get(Stat::SubStatPercent) / 100.0)
            + stats.get(Stat::SubStatPercentExempt))
        .floor();
        let atk_final =
            (stats.get(Stat::Atk) * (1.0 + stats.get(Stat::AtkPercent) / 100.0)).floor();
        let dmg_percent_final = stats.get(Stat::DmgPercent)
            + stats.get(Stat::BossDmgPercent)
            + stats.get(Stat::ExtraDmgPercent)
            + stats.get(Stat::ExtraBossDmgPercent);
        let ignore_guard_percent_final = add_ignore_guard_percents(vec![
            stats.get(Stat::IgnoreGuardPercent),
            stats.get(Stat::ExtraIgnoreGuardPercent),
        ]);
        let rate = stats.get(Stat::CritRatePercent) + stats.get(Stat::ExtraCritRatePercent);
        let crit_rate_final = if rate > 100.0 { 1.0 } else { rate / 100.0 };
        let crit_dmg_final =
            (stats.get(Stat::CritDmgPercent) + stats.get(Stat::ExtraCritDmgPercent)) / 100.0;

        let mut line_dmg = (main_stat_final * 4.0 + sub_stat_final) * 0.01;
        line_dmg = line_dmg * atk_final;
        line_dmg = line_dmg * stats.get(Stat::WeaponConstant) * stats.get(Stat::ClassConstant);
        line_dmg = line_dmg * (1.0 + dmg_percent_final / 100.0);
        line_dmg = line_dmg
            * (1.0 + stats.get(Stat::FinalDmgPercent) / 100.0)
            * (1.0 + stats.get(Stat::ExtraFinalDmgPercent) / 100.0);
        line_dmg = line_dmg
            * (1.0
                - (self.target_boss_guard_percent as f64) / 100.0
                    * (1.0 - ignore_guard_percent_final / 100.0));
        line_dmg = line_dmg * (crit_rate_final * (1.35 + crit_dmg_final) + (1.0 - crit_rate_final));
        line_dmg = line_dmg * (1.0 + stats.get(Stat::MasteryPercent) / 100.0) / 2.0;
        line_dmg = line_dmg
            * (1.0
                - ((self.target_boss_elemental_resistance_percent as f64) / 100.0)
                    * (1.0 - stats.get(Stat::IgnoreElementalResistancePercent) / 100.0));

        line_dmg
    }
}

#[derive(Debug, Clone)]
struct StatChange {
    stat: Stat,
    amount: f64,
}

impl StatChange {
    fn new(stat: Stat, amount: f64) -> Self {
        Self { stat, amount }
    }
}

#[derive(Debug, Clone)]
struct LinkSkill {
    name: String,
    effects: Vec<StatChange>,
}

impl LinkSkill {
    fn new(name: &str, effects: Vec<StatChange>) -> Self {
        Self {
            name: name.to_string(),
            effects,
        }
    }
}

fn link_skill_list() -> Vec<LinkSkill> {
    vec![
        LinkSkill::new(
            "Mage",
            vec![
                StatChange::new(Stat::ExtraDmgPercent, 9.0),
                StatChange::new(Stat::ExtraIgnoreGuardPercent, 9.0),
            ],
        ),
        LinkSkill::new("Archer", vec![StatChange::new(Stat::CritRatePercent, 10.0)]),
        LinkSkill::new("Thief", vec![StatChange::new(Stat::ExtraDmgPercent, 9.0)]),
        LinkSkill::new(
            "Pirate",
            vec![
                StatChange::new(Stat::MainStat, 70.0),
                StatChange::new(Stat::SubStat, 70.0),
            ],
        ),
        LinkSkill::new("Cygnus", vec![StatChange::new(Stat::Atk, 25.0)]),
        LinkSkill::new(
            "Xenon",
            vec![
                StatChange::new(Stat::MainStatPercent, 10.0),
                StatChange::new(Stat::SubStatPercent, 10.0),
            ],
        ),
        LinkSkill::new(
            "DemonSlayer",
            vec![StatChange::new(Stat::BossDmgPercent, 15.0)],
        ),
        LinkSkill::new(
            "DemonAvenger",
            vec![StatChange::new(Stat::DmgPercent, 10.0)],
        ),
        LinkSkill::new(
            "Luminous",
            vec![StatChange::new(Stat::IgnoreGuardPercent, 15.0)],
        ),
        LinkSkill::new(
            "Phantom",
            vec![StatChange::new(Stat::CritRatePercent, 15.0)],
        ),
        LinkSkill::new("Cain", vec![StatChange::new(Stat::ExtraDmgPercent, 8.5)]),
        LinkSkill::new("Cadena", vec![StatChange::new(Stat::ExtraDmgPercent, 12.0)]),
        LinkSkill::new(
            "Adel",
            vec![
                StatChange::new(Stat::DmgPercent, 8.0),
                StatChange::new(Stat::BossDmgPercent, 4.0),
            ],
        ),
        LinkSkill::new("Ilium", vec![StatChange::new(Stat::DmgPercent, 12.0)]),
        LinkSkill::new("Ark", vec![StatChange::new(Stat::DmgPercent, 11.0)]),
        LinkSkill::new(
            "Hoyoung",
            vec![StatChange::new(Stat::IgnoreGuardPercent, 10.0)],
        ),
        LinkSkill::new(
            "Zero",
            vec![StatChange::new(Stat::IgnoreGuardPercent, 10.0)],
        ),
        LinkSkill::new("Kinesis", vec![StatChange::new(Stat::CritDmgPercent, 4.0)]),
    ]
}

fn find_optimal_links(num: usize, stats: &Stats, info: CalculatorInfo) -> Vec<LinkSkill> {
    let mut current_stats = stats.clone();
    let mut link_list: Vec<(bool, LinkSkill)> =
        link_skill_list().into_iter().map(|l| (false, l)).collect();
    let mut result_list = vec![];
    println!("{:#?}", link_list);
    for _ in 0..num {
        let mut link_list_temp = link_list.clone();
        // get element where damage is highest
        let max_link = link_list_temp
            .iter_mut()
            .max_by_key(|(b, link)| {
                if *b {
                    OrderedFloat(-1.0)
                } else {
                    let new_stats = current_stats.get_stats_with_changes(link);
                    OrderedFloat(info.boss_line_dmg(new_stats))
                }
            })
            .unwrap();
        assert!(!max_link.0);
        max_link.0 = true;
        current_stats = current_stats.get_stats_with_changes(&max_link.1);
        result_list.push(max_link.1.clone());
        link_list = link_list_temp;
    }

    result_list
}

fn add_ignore_guard_percents(percents: Vec<f64>) -> f64 {
    let mut f = 1.0;
    for p in percents {
        f *= 1.0 - p / 100.0;
    }
    (1.0 - f) * 100.0
}

struct HyperStats {
    levels: Vec<(Stat, usize)>,
}

impl HyperStats {
    fn get_hyper_stat_points_from_level(level: usize) -> usize {
        assert!(level >= 140);
        assert!(level <= 275);
        let mut cursor = 140;
        let mut points = 0;
        while cursor <= level {
            points += cursor / 10 - 11;
            cursor += 1;
        }
        points
    }

    fn hyper_stats_info() -> Vec<(Stat, Vec<usize>)> {
        vec![
            (Stat::MainStatPercentExempt, vec![30; 15]),
            (Stat::SubStatPercentExempt, vec![30; 15]),
            (
                Stat::CritRatePercent,
                vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            ),
            (Stat::CritDmgPercent, vec![1; 15]),
            (Stat::IgnoreGuardPercent, vec![3; 15]),
            (Stat::DmgPercent, vec![3; 15]),
            (
                Stat::BossDmgPercent,
                vec![3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            ),
            (Stat::Atk, vec![3; 15]),
        ]
    }

    fn hyper_stats_combinations(info: &CalculatorInfo) -> Vec<Vec<usize>> {
        let hyper_stats = Self::hyper_stats_info();
        let mut points = Self::get_hyper_stat_points_from_level(info.level);
        let required_points_per_level =
            vec![1, 2, 4, 8, 10, 15, 20, 25, 30, 35, 50, 65, 80, 95, 110];
        for lvl in &info.used_hyper_stat_levels {
            points -= required_points_per_level[0..*lvl].iter().sum::<usize>();
        }
        println!("points: {}", points);
        let mut hyper_stat_levels = vec![0; 8];
        HyperStats::hyper_stats_dfs(
            &mut hyper_stat_levels,
            &required_points_per_level,
            &hyper_stats,
            0,
            points,
        )
    }

    fn hyper_stats_dfs(
        hyper_stat_levels: &mut Vec<usize>,
        required_points_per_level: &Vec<usize>,
        hyper_stats: &Vec<(Stat, Vec<usize>)>,
        current_index: usize,
        current_points: usize,
    ) -> Vec<Vec<usize>> {
        let mut points = current_points as isize;
        let mut combinations = vec![];
        if current_index >= hyper_stats.len() {
            return vec![hyper_stat_levels.clone()];
        }
        for i in 0..(required_points_per_level.len() + 1) {
            // 0..16
            hyper_stat_levels[current_index] = i;
            if i > 0 {
                points -= required_points_per_level[i - 1] as isize;
                if points < 0 {
                    break;
                }
                combinations.extend(Self::hyper_stats_dfs(
                    hyper_stat_levels,
                    required_points_per_level,
                    hyper_stats,
                    current_index + 1,
                    points as usize,
                ));
            }
        }

        combinations
    }
}

fn main() {
    let stats = Stats::init();
    let info = CalculatorInfo::init();
    // println!("{}", info.boss_line_dmg(stats) as u64);
    // println!("{:#?}", find_optimal_links(10, &stats, info));
    let links = link_skill_list();
    let link_combinations = links.iter().combinations(info.free_link_skill_spaces);
    let hyper_stats_combinations = HyperStats::hyper_stats_combinations(&info);
    println!("{}", hyper_stats_combinations.len());
}

#[cfg(test)]
mod tests {

    #[test]
    fn ignore_guard_test() {
        assert_eq!(super::add_ignore_guard_percents(vec![]), 0.0);
        assert_eq!(super::add_ignore_guard_percents(vec![30.0, 30.0]), 51.0);
    }

    #[test]
    fn hyper_stat_points_from_level_test() {
        assert_eq!(super::HyperStats::get_hyper_stat_points_from_level(140), 3);
        assert_eq!(
            super::HyperStats::get_hyper_stat_points_from_level(227),
            608
        );
        assert_eq!(
            super::HyperStats::get_hyper_stat_points_from_level(251),
            908
        );
        assert_eq!(
            super::HyperStats::get_hyper_stat_points_from_level(275),
            1266
        );
    }
}
