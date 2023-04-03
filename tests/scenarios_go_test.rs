#[test]
fn init_go() {
    multiversx_sc_scenario::run_go("scenarios/00_init.scen.json");
}

#[test]
fn setup_go() {
    multiversx_sc_scenario::run_go("scenarios/01_setup.scen.json");
}

#[test]
fn deposit_gng_go() {
    multiversx_sc_scenario::run_go("scenarios/02_deposit_gng.scen.json");
}

#[test]
fn stake_battle_one_go() {
    multiversx_sc_scenario::run_go("scenarios/03_stake_battle_one.scen.json");
}

#[test]
fn battle_one_go() {
    multiversx_sc_scenario::run_go("scenarios/04_battle_one.scen.json");
}

#[test]
fn claim_rewards_go() {
    multiversx_sc_scenario::run_go("scenarios/05_claim_rewards.scen.json");
}

#[test]
fn withdraw_go() {
    multiversx_sc_scenario::run_go("scenarios/06_withdraw.scen.json");
}

#[test]
fn battle_two_go() {
    multiversx_sc_scenario::run_go("scenarios/07_battle_two.scen.json");
}

#[test]
fn withdraw_and_claim_go() {
    multiversx_sc_scenario::run_go("scenarios/08_withdraw_and_claim.scen.json");
}

#[test]
fn stake_go() {
    multiversx_sc_scenario::run_go("scenarios/09_stake.scen.json");
}

#[test]
fn battle_three_go() {
    multiversx_sc_scenario::run_go("scenarios/10_battle_three.scen.json");
}

#[test]
fn withdraw_and_claim_two_go() {
    multiversx_sc_scenario::run_go("scenarios/11_withdraw_and_claim.scen.json");
}

#[test]
fn battle_four_and_five_go() {
    multiversx_sc_scenario::run_go("scenarios/12_battle_four_and_five.scen.json");
}

#[test]
fn stake_two_go() {
    multiversx_sc_scenario::run_go("scenarios/13_stake.scen.json");
}

#[test]
fn battle_six_go() {
    multiversx_sc_scenario::run_go("scenarios/14_battle_six.scen.json");
}

#[test]
fn change_attributes_go() {
    multiversx_sc_scenario::run_go("scenarios/15_change_attributes.scen.json");
}

#[test]
fn claim_rewards_during_battle_go() {
    multiversx_sc_scenario::run_go("scenarios/16_claim_rewards_during_battle_seven.scen.json");
}

#[test]
fn withdraw_not_yet_battled_go() {
    multiversx_sc_scenario::run_go("scenarios/16_withdraw_not_yet_battled_battle_seven.scen.json");
}

#[test]
fn withdraw_winner_go() {
    multiversx_sc_scenario::run_go("scenarios/16_withdraw_winner_while_battle_seven.scen.json");
}

#[test]
fn halving_go() {
    multiversx_sc_scenario::run_go("scenarios/test-halving.scen.json");
}

#[test]
fn is_today_special_go() {
    multiversx_sc_scenario::run_go("scenarios/test-is_today_special.scen.json");
}

#[test]
fn status_go() {
    multiversx_sc_scenario::run_go("scenarios/test-status.scen.json");
}
