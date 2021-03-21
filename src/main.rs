use ordered_float::OrderedFloat;

/// Members that end with `percent` are measured in units of %.  
/// ex) dmg_percent = 50 == DMG% = 50%  
/// Listed in X121:Y135 of Spec Simulator 0.9.4  
/// 메용 포함  
#[derive(Clone)]
struct Stats {
    main_stat: f64,                           // 주스탯
    main_stat_percent: f64,                   // 주스탯퍼
    main_stat_percent_exempt: f64,            // 스탯퍼 적용되지 않는 주스탯
    sub_stat: f64,                            // 부스탯
    sub_stat_percent: f64,                    // 부스탯퍼
    sub_stat_percent_exempt: f64,             // 스탯퍼 적용되지 않는 부스탯
    atk: f64,                                 // 공마
    atk_percent: f64,                         // 공마퍼
    dmg_percent: f64,                         // 뎀퍼
    boss_dmg_percent: f64,                    // 보공
    final_dmg_percent: f64,                   // 최종뎀
    ignore_guard_percent: f64,                // 방무
    ignore_elemental_resistance_percent: f64, // 속성내성무시
    crit_rate_percent: f64,                   // 크확
    crit_dmg_percent: f64,                    // 크뎀
    weapon_constant: f64,                     // 무기상수
    class_constant: f64,                      // 직업상수
    mastery_percent: f64,                     // 숙련도

    // 추가스탯; Y136:Y141
    extra_dmg_percent: f64,          // 추가뎀퍼
    extra_boss_dmg_percent: f64,     // 추가보공
    extra_final_dmg_percent: f64,    // 추가최종뎀
    extra_ignore_guard_percent: f64, // 추가방무
    extra_crit_rate_percent: f64,    // 추가크확
    extra_crit_dmg_percent: f64,     // 추가크뎀
}

impl Stats {
    fn init() -> Self {
        Self {
            main_stat: 4395 as f64,
            main_stat_percent: 331 as f64,
            main_stat_percent_exempt: 13780 as f64,
            sub_stat: 2014 as f64,
            sub_stat_percent: 103 as f64,
            sub_stat_percent_exempt: 440 as f64,
            atk: 1954 as f64,
            atk_percent: 97 as f64,
            dmg_percent: 57 as f64,
            boss_dmg_percent: 236 as f64,
            final_dmg_percent: 30 as f64,
            ignore_guard_percent: 78.73660272,
            ignore_elemental_resistance_percent: 5 as f64,
            crit_rate_percent: 70 as f64,
            crit_dmg_percent: 65 as f64,
            weapon_constant: 1.2,
            class_constant: 1.0,
            mastery_percent: 90 as f64,
            extra_dmg_percent: 3.2,
            extra_boss_dmg_percent: 0 as f64,
            extra_final_dmg_percent: 55.94,
            extra_ignore_guard_percent: 42.88,
            extra_crit_rate_percent: 0 as f64,
            extra_crit_dmg_percent: 0 as f64,
        }
    }

    fn display_stat_atk(&self) -> f64 {
        let main_stat_final = (self.main_stat * (1.0 + self.main_stat_percent / 100.0)
            + self.main_stat_percent_exempt)
            .floor();
        let sub_stat_final = (self.sub_stat * (1.0 + self.sub_stat_percent / 100.0)
            + self.sub_stat_percent_exempt)
            .floor();
        let atk_final = (self.atk * (1.0 + self.atk_percent / 100.0)).floor();

        let stat_atk = (main_stat_final * 4.0 + sub_stat_final) * 0.01;
        let stat_atk = stat_atk * atk_final;
        let stat_atk = stat_atk * self.weapon_constant * self.class_constant;
        let stat_atk = stat_atk * (1.0 + self.dmg_percent / 100.0);
        let stat_atk = stat_atk * (1.0 + self.final_dmg_percent / 100.0);

        stat_atk.floor()
    }

    fn get_stats_with_changes(&self, link: &LinkSkill) -> Self {
        let mut new_stats = self.clone();
        for effect in &link.effects {
            match effect {
                StatChange::AllStats(f) => {
                    new_stats.main_stat += f;
                    new_stats.sub_stat += f;
                }
                StatChange::AllStatsPercent(f) => {
                    new_stats.main_stat_percent += f;
                    new_stats.sub_stat_percent += f;
                }
                StatChange::Atk(f) => new_stats.atk += f,
                StatChange::BossDmgPercent(f) => new_stats.boss_dmg_percent += f,
                StatChange::CritDmgPercent(f) => new_stats.crit_dmg_percent += f,
                StatChange::CritRatePercent(f) => {
                    new_stats.crit_rate_percent += f;
                    if new_stats.crit_rate_percent > 100.0 {
                        new_stats.crit_rate_percent = 100.0;
                    }
                }
                StatChange::DmgPercent(f) => new_stats.dmg_percent += f,
                StatChange::ExtraDmgPercent(f) => new_stats.extra_dmg_percent += f,
                StatChange::ExtraIgnoreGuardPercent(f) => {
                    new_stats.extra_ignore_guard_percent =
                        add_ignore_guard_percents(vec![new_stats.extra_ignore_guard_percent, *f])
                }
                StatChange::IgnoreGuardPercent(f) => new_stats.ignore_guard_percent = add_ignore_guard_percents(vec![new_stats.ignore_guard_percent, *f])
            }
        }

        new_stats
    }
}

struct CalculatorInfo {
    target_boss_guard_percent: usize,                // 보스 방어율
    target_boss_elemental_resistance_percent: usize, // 보스 속성내성
}

impl CalculatorInfo {
    fn init() -> Self {
        Self {
            target_boss_guard_percent: 300,
            target_boss_elemental_resistance_percent: 50,
        }
    }

    /// 보스상대 한줄뎀  
    /// 렙차, 아케인포스 무시
    fn boss_line_dmg(&self, stats: Stats) -> f64 {
        let main_stat_final = (stats.main_stat * (1.0 + stats.main_stat_percent / 100.0)
            + stats.main_stat_percent_exempt)
            .floor();
        let sub_stat_final = (stats.sub_stat * (1.0 + stats.sub_stat_percent / 100.0)
            + stats.sub_stat_percent_exempt)
            .floor();
        let atk_final = (stats.atk * (1.0 + stats.atk_percent / 100.0)).floor();
        let dmg_percent_final = stats.dmg_percent
            + stats.boss_dmg_percent
            + stats.extra_dmg_percent
            + stats.extra_boss_dmg_percent;
        let ignore_guard_final = 1.0
            - (1.0 - stats.ignore_guard_percent / 100.0)
                * (1.0 - stats.extra_ignore_guard_percent / 100.0);
        let rate = stats.crit_rate_percent + stats.extra_crit_rate_percent;
        let crit_rate_final = if rate > 100.0 { 1.0 } else { rate / 100.0 };
        let crit_dmg_final = (stats.crit_dmg_percent + stats.extra_crit_dmg_percent) / 100.0;

        let mut line_dmg = (main_stat_final * 4.0 + sub_stat_final) * 0.01;
        line_dmg = line_dmg * atk_final;
        line_dmg = line_dmg * stats.weapon_constant * stats.class_constant;
        line_dmg = line_dmg * (1.0 + dmg_percent_final / 100.0);
        line_dmg = line_dmg
            * (1.0 + stats.final_dmg_percent / 100.0)
            * (1.0 + stats.extra_final_dmg_percent / 100.0);
        line_dmg = line_dmg
            * (1.0 - (self.target_boss_guard_percent as f64) / 100.0 * (1.0 - ignore_guard_final));
        line_dmg = line_dmg * (crit_rate_final * (1.35 + crit_dmg_final) + (1.0 - crit_rate_final));
        line_dmg = line_dmg * (1.0 + stats.mastery_percent / 100.0) / 2.0;
        line_dmg = line_dmg
            * (1.0
                - ((self.target_boss_elemental_resistance_percent as f64) / 100.0)
                    * (1.0 - stats.ignore_elemental_resistance_percent / 100.0));

        line_dmg
    }
}

#[derive(Debug, Clone)]
enum StatChange {
    DmgPercent(f64),
    IgnoreGuardPercent(f64),
    CritRatePercent(f64),
    AllStats(f64),
    Atk(f64),
    AllStatsPercent(f64),
    BossDmgPercent(f64),
    CritDmgPercent(f64),
    ExtraDmgPercent(f64),
    ExtraIgnoreGuardPercent(f64),
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
                StatChange::ExtraDmgPercent(9.0),
                StatChange::ExtraIgnoreGuardPercent(9.0),
            ],
        ),
        LinkSkill::new("Archer", vec![StatChange::CritRatePercent(10.0)]),
        LinkSkill::new("Thief", vec![StatChange::ExtraDmgPercent(9.0)]),
        LinkSkill::new("Pirate", vec![StatChange::AllStats(70.0)]),
        LinkSkill::new("Cygnus", vec![StatChange::Atk(25.0)]),
        LinkSkill::new("Xenon", vec![StatChange::AllStatsPercent(10.0)]),
        LinkSkill::new("DemonSlayer", vec![StatChange::BossDmgPercent(15.0)]),
        LinkSkill::new("DemonAvenger", vec![StatChange::DmgPercent(10.0)]),
        LinkSkill::new("Luminous", vec![StatChange::IgnoreGuardPercent(15.0)]),
        LinkSkill::new("Phantom", vec![StatChange::CritRatePercent(15.0)]),
        LinkSkill::new("Cain", vec![StatChange::ExtraDmgPercent(8.5)]),
        LinkSkill::new("Cadena", vec![StatChange::ExtraDmgPercent(12.0)]),
        LinkSkill::new(
            "Adel",
            vec![StatChange::DmgPercent(8.0), StatChange::BossDmgPercent(4.0)],
        ),
        LinkSkill::new("Ilium", vec![StatChange::DmgPercent(12.0)]),
        LinkSkill::new("Ark", vec![StatChange::DmgPercent(11.0)]),
        LinkSkill::new("Hoyoung", vec![StatChange::IgnoreGuardPercent(10.0)]),
        LinkSkill::new("Zero", vec![StatChange::IgnoreGuardPercent(10.0)]),
        LinkSkill::new("Kinesis", vec![StatChange::CritDmgPercent(4.0)]),
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
        let max_link = link_list_temp.iter_mut().max_by_key(|(b, link)| {
            if *b {
                OrderedFloat(-1.0)
            } else {
                let new_stats = current_stats.get_stats_with_changes(link);
                OrderedFloat(info.boss_line_dmg(new_stats))
            }
        }).unwrap();
        assert!(!max_link.0);
        max_link.0 = true;
        current_stats = current_stats.get_stats_with_changes(&max_link.1);
        result_list.push(max_link.1.clone());
        link_list = link_list_temp;
    }

    println!("{:?}", result_list);

    result_list
}

fn add_ignore_guard_percents(percents: Vec<f64>) -> f64 {
    let mut f = 1.0;
    for p in percents {
        f *= 1.0 - p / 100.0;
    }
    (1.0 - f) * 100.0
}

fn main() {
    let stats = Stats::init();
    let info = CalculatorInfo::init();
    // println!("{}", info.boss_line_dmg(stats) as u64);
    println!("{:#?}", find_optimal_links(10, &stats, info));
}

#[cfg(test)]
mod tests {

    #[test]
    fn ignore_guard_test() {
        assert_eq!(super::add_ignore_guard_percents(vec![]), 0.0);
    }

    #[test]
    fn ignore_guard_test_2() {
        assert_eq!(super::add_ignore_guard_percents(vec![30.0, 30.0]), 51.0);
    }
}
