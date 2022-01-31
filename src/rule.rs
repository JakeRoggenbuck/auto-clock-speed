// We will see if this is useful
// It might not be because of how hard holding state will be

pub trait ChargingActions {
    fn run(&self);
    fn check(&self, charging: bool, already_charging: bool) -> bool;
    fn get_name(&self) -> &str;
    fn get_docs(&self) -> &str;
}

pub struct StartChargingRule {
    pub name: String,
    pub docs: String,
}

pub struct EndChargingRule {
    pub name: String,
    pub docs: String,
}

pub struct LidCloseRule {
    pub name: String,
    pub docs: String,
}

pub struct LidOpenRule {
    pub name: String,
    pub docs: String,
}

pub struct UnderPowersaveUnderRule {
    pub name: String,
    pub docs: String,
}

impl ChargingActions for StartChargingRule {
    fn run(&self) {}
    fn check(&self, charging: bool, already_charging: bool) -> bool {
        charging && !already_charging
    }
    fn get_name(&self) -> &str {
        "StartChargingRule"
    }
    fn get_docs(&self) -> &str {
        "Check if AC connects"
    }
}

impl ChargingActions for EndChargingRule {
    fn run(&self) {}
    fn check(&self, charging: bool, already_charging: bool) -> bool {
        !charging && already_charging
    }
    fn get_name(&self) -> &str {
        "EndChargingRule"
    }
    fn get_docs(&self) -> &str {
        "Check if AC disconnects"
    }
}
