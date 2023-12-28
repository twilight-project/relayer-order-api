// use crate::db::*;
use crate::relayer::*;
use serde_derive::{Deserialize, Serialize};
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
    CreateTraderOrder(CreateTraderOrder, Meta),
    CreateLendOrder(CreateLendOrder, Meta),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta),
    ExecuteLendOrder(ExecuteLendOrder, Meta),
    CancelTraderOrder(CancelTraderOrder, Meta),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}
