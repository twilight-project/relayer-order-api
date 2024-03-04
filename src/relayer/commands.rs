use crate::relayer::*;
use serde_derive::{Deserialize, Serialize};
pub type ZkosHexString = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RpcCommand {
    CreateTraderOrder(CreateTraderOrder, Meta, ZkosHexString),
    CreateLendOrder(CreateLendOrder, Meta, ZkosHexString),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta, ZkosHexString),
    ExecuteLendOrder(ExecuteLendOrder, Meta, ZkosHexString),
    CancelTraderOrder(CancelTraderOrder, Meta, ZkosHexString),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}
