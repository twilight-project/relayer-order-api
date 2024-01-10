// use crate::db::*;
use crate::relayer::*;
use serde_derive::{Deserialize, Serialize};
pub type Zkos_Hex_String = String;
// use uuid::Uuid;

// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub enum RelayerCommand {
//     FundingCycle(PoolBatchOrder, Meta),
//     PriceTickerLiquidation(PoolBatchOrder, Meta),
//     PriceTickerOrderFill(PoolBatchOrder, Meta), //no update for lend pool
//     PriceTickerOrderSettle(PoolBatchOrder, Meta),
//     FundingCycleLiquidation(PoolBatchOrder, Meta),
//     RpcCommandPoolupdate(PoolBatchOrder, Meta),
//     InitiateNewPool(LendOrder, Meta),
//     AddTraderOrderToBatch(TraderOrder, RpcCommand, Meta, f64),
// }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RpcCommand {
    CreateTraderOrder(CreateTraderOrder, Meta, Zkos_Hex_String),
    CreateLendOrder(CreateLendOrder, Meta, Zkos_Hex_String),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta, Zkos_Hex_String),
    ExecuteLendOrder(ExecuteLendOrder, Meta, Zkos_Hex_String),
    CancelTraderOrder(CancelTraderOrder, Meta, Zkos_Hex_String),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}
