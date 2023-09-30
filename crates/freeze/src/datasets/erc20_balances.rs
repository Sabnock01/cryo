use crate::{
    dataframes::SortableDataFrame, store, with_series, with_series_binary, with_series_option_u256,
    CollectByBlock, CollectError, ColumnData, ColumnEncoding, ColumnType, Dataset, Datatype,
    Erc20Balances, Params, Schemas, Source, Table, ToVecHex, ToVecU8, U256Type,
};
use ethers::prelude::*;
use polars::prelude::*;
use std::collections::HashMap;

/// columns for transactions
#[cryo_to_df::to_df(Datatype::Erc20Balances)]
#[derive(Default)]
pub struct Erc20BalancesColumns {
    n_rows: u64,
    block_number: Vec<u32>,
    erc20: Vec<Vec<u8>>,
    address: Vec<Vec<u8>>,
    balance: Vec<Option<U256>>,
}

#[async_trait::async_trait]
impl Dataset for Erc20Balances {
    fn datatype(&self) -> Datatype {
        Datatype::Erc20Balances
    }

    fn name(&self) -> &'static str {
        "erc20_balances"
    }

    fn column_types(&self) -> HashMap<&'static str, ColumnType> {
        HashMap::from_iter(vec![
            ("block_number", ColumnType::UInt32),
            ("erc20", ColumnType::Binary),
            ("address", ColumnType::Binary),
            ("balance", ColumnType::UInt256),
            ("chain_id", ColumnType::UInt64),
        ])
    }

    fn default_columns(&self) -> Vec<&'static str> {
        vec!["block_number", "erc20", "address", "balance", "chain_id"]
    }

    fn default_sort(&self) -> Vec<String> {
        vec!["block_number".to_string()]
    }
}

type Result<T> = ::core::result::Result<T, CollectError>;

type BlockErc20AddressBalance = (u32, Vec<u8>, Vec<u8>, Option<U256>);

#[async_trait::async_trait]
impl CollectByBlock for Erc20Balances {
    type Response = BlockErc20AddressBalance;

    type Columns = Erc20BalancesColumns;

    async fn extract(request: Params, source: Source, _schemas: Schemas) -> Result<Self::Response> {
        let signature: Vec<u8> = prefix_hex::decode("0x70a08231").expect("Decoding failed");
        let mut call_data = signature.clone();
        call_data.extend(request.address());
        let block_number = request.ethers_block_number();
        let contract = request.ethers_contract();
        let balance = source.fetcher.call2(contract, call_data, block_number).await.ok();
        let balance = balance.map(|x| x.to_vec().as_slice().into());
        Ok((request.block_number() as u32, request.contract(), request.address(), balance))
    }

    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        let schema = schemas.get(&Datatype::Erc20Balances).expect("missing schema");
        let (block, erc20, address, balance) = response;
        columns.n_rows += 1;
        store!(schema, columns, block_number, block);
        store!(schema, columns, erc20, erc20);
        store!(schema, columns, address, address);
        store!(schema, columns, balance, balance);
    }
}
