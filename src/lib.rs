mod abi;
mod pb;
mod rpc;
use hex_literal::hex;
use pb::contract::v1 as contract;
use substreams::prelude::*;
use substreams::store;
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_entity_change::tables::Tables as EntityChangesTables;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;
use std::str::FromStr;
use substreams::scalar::BigDecimal;

substreams_ethereum::init!();

const FACTORY_TRACKED_CONTRACT: [u8; 20] = hex!("e00384587dc733d1e201e1eaa5583645d351c01c");

fn map_factory_events(blk: &eth::Block, events: &mut contract::Events) {
    events.factory_admin_changeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::AdminChanged::match_and_decode(log) {
                        return Some(contract::FactoryAdminChanged {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            new_admin: event.new_admin,
                            previous_admin: event.previous_admin,
                        });
                    }

                    None
                })
        })
        .collect());
    events.factory_beacon_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::BeaconUpgraded::match_and_decode(log) {
                        return Some(contract::FactoryBeaconUpgraded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            beacon: event.beacon,
                        });
                    }

                    None
                })
        })
        .collect());
    events.factory_deployed_lender_group_contracts.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::DeployedLenderGroupContract::match_and_decode(log) {
                        return Some(contract::FactoryDeployedLenderGroupContract {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            group_contract: event.group_contract,
                        });
                    }

                    None
                })
        })
        .collect());
    events.factory_upgradeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::factory_contract::events::Upgraded::match_and_decode(log) {
                        return Some(contract::FactoryUpgraded {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            implementation: event.implementation,
                        });
                    }

                    None
                })
        })
        .collect());
}

fn is_declared_dds_address(addr: &Vec<u8>, ordinal: u64, dds_store: &store::StoreGetInt64) -> bool {
    //    substreams::log::info!("Checking if address {} is declared dds address", Hex(addr).to_string());
    if dds_store.get_at(ordinal, Hex(addr).to_string()).is_some() {
        return true;
    }
    return false;
}

fn map_lendergroup_events(
    blk: &eth::Block,
    dds_store: &store::StoreGetInt64,
    events: &mut contract::Events,
) {

    events.lendergroup_borrower_accepted_funds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::BorrowerAcceptedFunds::match_and_decode(log) {
                        return Some(contract::LendergroupBorrowerAcceptedFunds {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            bid_id: event.bid_id.to_string(),
                            borrower: event.borrower,
                            collateral_amount: event.collateral_amount.to_string(),
                            interest_rate: event.interest_rate.to_u64(),
                            loan_duration: event.loan_duration.to_u64(),
                            principal_amount: event.principal_amount.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_defaulted_loan_liquidateds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::DefaultedLoanLiquidated::match_and_decode(log) {
                        return Some(contract::LendergroupDefaultedLoanLiquidated {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount_due: event.amount_due.to_string(),
                            bid_id: event.bid_id.to_string(),
                            liquidator: event.liquidator,
                            token_amount_difference: event.token_amount_difference.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_earnings_withdrawns.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::EarningsWithdrawn::match_and_decode(log) {
                        return Some(contract::LendergroupEarningsWithdrawn {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount_pool_shares_tokens: event.amount_pool_shares_tokens.to_string(),
                            lender: event.lender,
                            principal_tokens_withdrawn: event.principal_tokens_withdrawn.to_string(),
                            recipient: event.recipient,
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_initializeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::Initialized::match_and_decode(log) {
                        return Some(contract::LendergroupInitialized {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            version: event.version.to_u64(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_lender_added_principals.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::LenderAddedPrincipal::match_and_decode(log) {
                        return Some(contract::LendergroupLenderAddedPrincipal {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            amount: event.amount.to_string(),
                            lender: event.lender,
                            shares_amount: event.shares_amount.to_string(),
                            shares_recipient: event.shares_recipient,
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_loan_repaids.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::LoanRepaid::match_and_decode(log) {
                        return Some(contract::LendergroupLoanRepaid {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            bid_id: event.bid_id.to_string(),
                            interest_amount: event.interest_amount.to_string(),
                            principal_amount: event.principal_amount.to_string(),
                            repayer: event.repayer,
                            total_interest_collected: event.total_interest_collected.to_string(),
                            total_principal_repaid: event.total_principal_repaid.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_ownership_transferreds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::OwnershipTransferred::match_and_decode(log) {
                        return Some(contract::LendergroupOwnershipTransferred {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            new_owner: event.new_owner,
                            previous_owner: event.previous_owner,
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_pauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::Paused::match_and_decode(log) {
                        return Some(contract::LendergroupPaused {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            account: event.account,
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_pool_initializeds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::PoolInitialized::match_and_decode(log) {
                        
                        
                       let lender_group_contract_address = Hex(&log.address).to_string();

                        let fetched_rpc_data = rpc::fetch_lender_group_pool_initialization_data_from_rpc(
                            &lender_group_contract_address
                             ).unwrap();

                        
                        return Some(contract::LendergroupPoolInitialized {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            collateral_token_address: event.collateral_token_address,
                            interest_rate_lower_bound: event.interest_rate_lower_bound.to_u64(),
                            interest_rate_upper_bound: event.interest_rate_upper_bound.to_u64(),
                            liquidity_threshold_percent: event.liquidity_threshold_percent.to_u64(),
                            loan_to_value_percent: event.loan_to_value_percent.to_u64(),
                            market_id: event.market_id.to_string(),
                            max_loan_duration: event.max_loan_duration.to_u64(),
                            pool_shares_token: event.pool_shares_token,
                            principal_token_address: event.principal_token_address,
                            twap_interval: event.twap_interval.to_u64(),
                            uniswap_pool_fee: event.uniswap_pool_fee.to_u64(),

                            teller_v2_address: fetched_rpc_data.teller_v2_address.to_fixed_bytes().to_vec(),
                            uniswap_v3_pool_address: fetched_rpc_data.uniswap_v3_pool_address.to_fixed_bytes().to_vec(),
                            smart_commitment_forwarder_address: fetched_rpc_data.smart_commitment_forwarder_address.to_fixed_bytes().to_vec(),
                        });
                    }

                    None
                })
        })
        .collect());

    events.lendergroup_unpauseds.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| is_declared_dds_address(&log.address, log.ordinal, dds_store))
                .filter_map(|log| {
                    if let Some(event) = abi::lendergroup_contract::events::Unpaused::match_and_decode(log) {
                        return Some(contract::LendergroupUnpaused {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: blk.timestamp_seconds(),
                            evt_block_number: blk.number,
                            evt_address: Hex(&log.address).to_string(),
                            account: event.account,
                        });
                    }

                    None
                })
        })
        .collect());
}


fn db_factory_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.factory_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("factory_admin_changed", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time )
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin", Hex(&evt.new_admin).to_string())
            .set("previous_admin", Hex(&evt.previous_admin).to_string());
    });
    events.factory_beacon_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("factory_beacon_upgraded", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("beacon", Hex(&evt.beacon).to_string());
    });
    events.factory_deployed_lender_group_contracts.iter().for_each(|evt| {
        tables
            .create_row("factory_deployed_lender_group_contract", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("group_contract", Hex(&evt.group_contract).to_string());
    });
    events.factory_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("factory_upgraded", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("implementation", Hex(&evt.implementation).to_string());
    });
}
fn db_lendergroup_out(events: &contract::Events, tables: &mut DatabaseChangeTables) {
    // Loop over all the abis events to create table changes
    events.lendergroup_borrower_accepted_funds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_borrower_accepted_funds", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("borrower", Hex(&evt.borrower).to_string())
            .set("collateral_amount", BigDecimal::from_str(&evt.collateral_amount).unwrap())
            .set("interest_rate", evt.interest_rate)
            .set("loan_duration", evt.loan_duration)
            .set("principal_amount", BigDecimal::from_str(&evt.principal_amount).unwrap());
    });
    events.lendergroup_defaulted_loan_liquidateds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_defaulted_loan_liquidated", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount_due", BigDecimal::from_str(&evt.amount_due).unwrap())
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("liquidator", Hex(&evt.liquidator).to_string())
            .set("token_amount_difference", BigDecimal::from_str(&evt.token_amount_difference).unwrap());
    });
    events.lendergroup_earnings_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_earnings_withdrawn", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount_pool_shares_tokens", BigDecimal::from_str(&evt.amount_pool_shares_tokens).unwrap())
            .set("lender", Hex(&evt.lender).to_string())
            .set("principal_tokens_withdrawn", BigDecimal::from_str(&evt.principal_tokens_withdrawn).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string());
    });
    events.lendergroup_initializeds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_initialized", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("version", evt.version);
    });
    events.lendergroup_lender_added_principals.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_lender_added_principal", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("lender", Hex(&evt.lender).to_string())
            .set("shares_amount", BigDecimal::from_str(&evt.shares_amount).unwrap())
            .set("shares_recipient", Hex(&evt.shares_recipient).to_string());
    });
    events.lendergroup_loan_repaids.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_loan_repaid", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("interest_amount", BigDecimal::from_str(&evt.interest_amount).unwrap())
            .set("principal_amount", BigDecimal::from_str(&evt.principal_amount).unwrap())
            .set("repayer", Hex(&evt.repayer).to_string())
            .set("total_interest_collected", BigDecimal::from_str(&evt.total_interest_collected).unwrap())
            .set("total_principal_repaid", BigDecimal::from_str(&evt.total_principal_repaid).unwrap());
    });
    events.lendergroup_ownership_transferreds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_ownership_transferred", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("new_owner", Hex(&evt.new_owner).to_string())
            .set("previous_owner", Hex(&evt.previous_owner).to_string());
    });
    events.lendergroup_pauseds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_paused", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("account", Hex(&evt.account).to_string());
    });
    events.lendergroup_pool_initializeds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_pool_initialized", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("collateral_token_address", Hex(&evt.collateral_token_address).to_string())
            .set("interest_rate_lower_bound", evt.interest_rate_lower_bound)
            .set("interest_rate_upper_bound", evt.interest_rate_upper_bound)
            .set("liquidity_threshold_percent", evt.liquidity_threshold_percent)
            .set("loan_to_value_percent", evt.loan_to_value_percent)
            .set("market_id", BigDecimal::from_str(&evt.market_id).unwrap())
            .set("max_loan_duration", evt.max_loan_duration)
            .set("pool_shares_token", Hex(&evt.pool_shares_token).to_string())
            .set("principal_token_address", Hex(&evt.principal_token_address).to_string())
            .set("twap_interval", evt.twap_interval)
            .set("uniswap_pool_fee", evt.uniswap_pool_fee);
    });
    events.lendergroup_unpauseds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_unpaused", [("evt_tx_hash", evt.evt_tx_hash.to_string()),("evt_index", evt.evt_index.to_string())])
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("account", Hex(&evt.account).to_string());
    });
}


fn graph_factory_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.factory_admin_changeds.iter().for_each(|evt| {
        tables
            .create_row("factory_admin_changed", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("new_admin", Hex(&evt.new_admin).to_string())
            .set("previous_admin", Hex(&evt.previous_admin).to_string());
    });
    events.factory_beacon_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("factory_beacon_upgraded", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("beacon", Hex(&evt.beacon).to_string());
    });
    events.factory_deployed_lender_group_contracts.iter().for_each(|evt| {
        tables
            .create_row("factory_deployed_lender_group_contract", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("group_contract", Hex(&evt.group_contract).to_string());
    });
    events.factory_upgradeds.iter().for_each(|evt| {
        tables
            .create_row("factory_upgraded", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("implementation", Hex(&evt.implementation).to_string());
    });
}
fn graph_lendergroup_out(events: &contract::Events, tables: &mut EntityChangesTables) {
    // Loop over all the abis events to create table changes
    events.lendergroup_borrower_accepted_funds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_borrower_accepted_funds", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("borrower", Hex(&evt.borrower).to_string())
            .set("collateral_amount", BigDecimal::from_str(&evt.collateral_amount).unwrap())
            .set("interest_rate", evt.interest_rate)
            .set("loan_duration", evt.loan_duration)
            .set("principal_amount", BigDecimal::from_str(&evt.principal_amount).unwrap());
    });
    events.lendergroup_defaulted_loan_liquidateds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_defaulted_loan_liquidated", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount_due", BigDecimal::from_str(&evt.amount_due).unwrap())
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("liquidator", Hex(&evt.liquidator).to_string())
            .set("token_amount_difference", BigDecimal::from_str(&evt.token_amount_difference).unwrap());
    });
    events.lendergroup_earnings_withdrawns.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_earnings_withdrawn", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount_pool_shares_tokens", BigDecimal::from_str(&evt.amount_pool_shares_tokens).unwrap())
            .set("lender", Hex(&evt.lender).to_string())
            .set("principal_tokens_withdrawn", BigDecimal::from_str(&evt.principal_tokens_withdrawn).unwrap())
            .set("recipient", Hex(&evt.recipient).to_string());
    });
    events.lendergroup_initializeds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_initialized", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("version", evt.version);
    });
    events.lendergroup_lender_added_principals.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_lender_added_principal", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("amount", BigDecimal::from_str(&evt.amount).unwrap())
            .set("lender", Hex(&evt.lender).to_string())
            .set("shares_amount", BigDecimal::from_str(&evt.shares_amount).unwrap())
            .set("shares_recipient", Hex(&evt.shares_recipient).to_string());
    });
    events.lendergroup_loan_repaids.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_loan_repaid", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("bid_id", BigDecimal::from_str(&evt.bid_id).unwrap())
            .set("interest_amount", BigDecimal::from_str(&evt.interest_amount).unwrap())
            .set("principal_amount", BigDecimal::from_str(&evt.principal_amount).unwrap())
            .set("repayer", Hex(&evt.repayer).to_string())
            .set("total_interest_collected", BigDecimal::from_str(&evt.total_interest_collected).unwrap())
            .set("total_principal_repaid", BigDecimal::from_str(&evt.total_principal_repaid).unwrap());
    });
    events.lendergroup_ownership_transferreds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_ownership_transferred", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("new_owner", Hex(&evt.new_owner).to_string())
            .set("previous_owner", Hex(&evt.previous_owner).to_string());
    });
    events.lendergroup_pauseds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_paused", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("account", Hex(&evt.account).to_string());
    });
    events.lendergroup_pool_initializeds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_pool_initialized", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("collateral_token_address", Hex(&evt.collateral_token_address).to_string())
            .set("interest_rate_lower_bound", evt.interest_rate_lower_bound)
            .set("interest_rate_upper_bound", evt.interest_rate_upper_bound)
            .set("liquidity_threshold_percent", evt.liquidity_threshold_percent)
            .set("loan_to_value_percent", evt.loan_to_value_percent)
            .set("market_id", BigDecimal::from_str(&evt.market_id).unwrap())
            .set("max_loan_duration", evt.max_loan_duration)
            .set("pool_shares_token", Hex(&evt.pool_shares_token).to_string())
            .set("principal_token_address", Hex(&evt.principal_token_address).to_string())
            .set("twap_interval", evt.twap_interval)
            .set("uniswap_pool_fee", evt.uniswap_pool_fee);
    });
    events.lendergroup_unpauseds.iter().for_each(|evt| {
        tables
            .create_row("lendergroup_unpaused", format!("{}-{}", evt.evt_tx_hash, evt.evt_index))
            .set("evt_tx_hash", &evt.evt_tx_hash)
            .set("evt_index", evt.evt_index)
            .set("evt_block_time", evt.evt_block_time)
            .set("evt_block_number", evt.evt_block_number)
            .set("evt_address", &evt.evt_address)
            .set("account", Hex(&evt.account).to_string());
    });
}
#[substreams::handlers::store]
fn store_factory_lendergroup_created(blk: eth::Block, store: StoreSetInt64) {
    for rcpt in blk.receipts() {
        for log in rcpt
            .receipt
            .logs
            .iter()
            .filter(|log| log.address == FACTORY_TRACKED_CONTRACT)
        {
            if let Some(event) = abi::factory_contract::events::DeployedLenderGroupContract::match_and_decode(log) {
                store.set(log.ordinal, Hex(event.group_contract).to_string(), &1);
            }
        }
    }
}

#[substreams::handlers::map]
fn map_events(
    blk: eth::Block,
    store_lendergroup: StoreGetInt64,
) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_factory_events(&blk, &mut events);
    map_lendergroup_events(&blk, &store_lendergroup, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = DatabaseChangeTables::new();
    db_factory_out(&events, &mut tables);
    db_lendergroup_out(&events, &mut tables);
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
fn graph_out(events: contract::Events) -> Result<EntityChanges, substreams::errors::Error> {
    // Initialize Database Changes container
    let mut tables = EntityChangesTables::new();
    graph_factory_out(&events, &mut tables);
    graph_lendergroup_out(&events, &mut tables);
    Ok(tables.to_entity_changes())
}
