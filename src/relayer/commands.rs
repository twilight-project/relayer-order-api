use crate::relayer::*;
use serde_derive::{Deserialize, Serialize};
pub type ZkosHexString = String;
pub type RequestId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RpcCommand {
    CreateTraderOrder(CreateTraderOrder, Meta, ZkosHexString, RequestId),
    CreateLendOrder(CreateLendOrder, Meta, ZkosHexString, RequestId),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta, ZkosHexString, RequestId),
    ExecuteLendOrder(ExecuteLendOrder, Meta, ZkosHexString, RequestId),
    CancelTraderOrder(CancelTraderOrder, Meta, ZkosHexString, RequestId),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}
