/// Members that end with `percent` and `ignore_guard` are measured in units of %.  
/// ex) dmg_percent = 50 == DMG% = 50%  
/// Listed in X121:Y135 of Spec Simulator 0.9.4  
/// 메용 포함  
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
    crit_percent: f64,                        // 크확
    crit_dmg_percent: f64,                    // 크뎀
    weapon_constant: f64,                     // 무기상수
    class_constant: f64,                      // 직업상수

    // 추가스탯; Y136:Y141
    extra_dmg_percent: f64,          // 추가뎀퍼
    extra_boss_dmg_percent: f64,     // 추가보공
    extra_final_dmg_percent: f64,    // 추가최종뎀
    extra_ignore_guard_percent: f64, // 추가방무
    extra_crit_percent: f64,         // 추가크확
    extra_crit_dmg_percent: f64,     // 추가크뎀
}

struct CalculatorInfo {
    target_boss_guard: usize, // 보스 방어율
}

impl Stats {
    fn init() -> Self {
        Self {
            main_stat: 4395 as f64,
            main_stat_percent: 331 as f64,
            main_stat_percent_exempt: 13870 as f64,
            sub_stat: 2014 as f64,
            sub_stat_percent: 103 as f64,
            sub_stat_percent_exempt: 530 as f64,
            atk: 1972 as f64,
            atk_percent: 97 as f64,
            dmg_percent: 87 as f64,
            boss_dmg_percent: 275 as f64,
            final_dmg_percent: 30 as f64,
            ignore_guard_percent: 78.73660272,
            ignore_elemental_resistance_percent: 5 as f64,
            crit_percent: 70 as f64,
            crit_dmg_percent: 65 as f64,
            weapon_constant: 1.2,
            class_constant: 1.0,
            extra_dmg_percent: 42.2,
            extra_boss_dmg_percent: 0 as f64,
            extra_final_dmg_percent: 55.94,
            extra_ignore_guard_percent: 48.0208,
            extra_crit_percent: 0 as f64,
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

        stat_atk
    }
}

impl CalculatorInfo {
    fn init() -> Self {
        Self {
            target_boss_guard: 300,
        }
    }
}

fn main() {
    let stats = Stats::init();
    let info = CalculatorInfo::init();
    println!("{}", stats.display_stat_atk().floor() as u64);
}
