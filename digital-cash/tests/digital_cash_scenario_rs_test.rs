use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("digital-cash");

    blockchain.register_contract(
        "file:output/digital-cash.wasm",
        digital_cash::ContractBuilder,
    );
    blockchain
}

#[test]
fn claim_rewa_rs() {
    world().run("scenarios/claim-rewa.scen.json");
}

#[test]
fn claim_dcdt_rs() {
    world().run("scenarios/claim-dcdt.scen.json");
}

#[test]
fn claim_fees_rs() {
    world().run("scenarios/claim-fees.scen.json");
}

#[test]
fn claim_multi_dcdt_rs() {
    world().run("scenarios/claim-multi-dcdt.scen.json");
}

#[test]
fn forward_rs() {
    world().run("scenarios/forward.scen.json");
}

#[test]
fn fund_rewa_and_dcdt_rs() {
    world().run("scenarios/fund-rewa-and-dcdt.scen.json");
}

#[test]
fn set_accounts_rs() {
    world().run("scenarios/set-accounts.scen.json");
}

#[test]
fn withdraw_rewa_rs() {
    world().run("scenarios/withdraw-rewa.scen.json");
}

#[test]
fn withdraw_dcdt_rs() {
    world().run("scenarios/withdraw-dcdt.scen.json");
}

#[test]
fn withdraw_multi_dcdt_rs() {
    world().run("scenarios/withdraw-multi-dcdt.scen.json");
}
