use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn claim_rewa_go() {
    world().run("scenarios/claim-rewa.scen.json");
}

#[test]
fn claim_dcdt_go() {
    world().run("scenarios/claim-dcdt.scen.json");
}

#[test]
fn claim_fees_go() {
    world().run("scenarios/claim-fees.scen.json");
}

#[test]
fn claim_multi_dcdt_go() {
    world().run("scenarios/claim-multi-dcdt.scen.json");
}

#[test]
fn forward_go() {
    world().run("scenarios/forward.scen.json");
}

#[test]
fn fund_rewa_and_dcdt_go() {
    world().run("scenarios/fund-rewa-and-dcdt.scen.json");
}

#[test]
fn set_accounts_go() {
    world().run("scenarios/set-accounts.scen.json");
}

#[test]
fn withdraw_rewa_go() {
    world().run("scenarios/withdraw-rewa.scen.json");
}

#[test]
fn withdraw_dcdt_go() {
    world().run("scenarios/withdraw-dcdt.scen.json");
}

#[test]
fn withdraw_multi_dcdt_go() {
    world().run("scenarios/withdraw-multi-dcdt.scen.json");
}
