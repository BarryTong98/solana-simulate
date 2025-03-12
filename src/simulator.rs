use {
    base64::{prelude::BASE64_STANDARD, Engine},
    serde_json,
    solana_bpf_loader_program::syscalls::create_program_runtime_environment_v1,
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_program_runtime::{
        __private::{Hash, ReadableAccount},
        loaded_programs::{
            BlockRelation,
            ForkGraph,
            LoadProgramMetrics,
            ProgramCacheEntry,
            ProgramRuntimeEnvironments,
        },
        solana_sbpf::{
            program::BuiltinProgram,
            vm::Config,
        },
    },
    solana_sdk::{
        account::{Account, AccountSharedData},
        clock::{Clock, Slot, UnixTimestamp, MAX_PROCESSING_AGE, MAX_TRANSACTION_FORWARDING_DELAY},
        feature_set::FeatureSet,
        inner_instruction::InnerInstructions,
        message::{AccountKeys, AddressLoader, AddressLoaderError},
        message::v0::{LoadedAddresses, MessageAddressTableLookup},
        native_loader,
        nonce::state::DurableNonce,
        pubkey::Pubkey,
        sysvar::SysvarId,
        transaction::{SanitizedTransaction, TransactionError},
        transaction_context::{TransactionAccount, TransactionReturnData},
    },
    solana_svm::{
        account_loader::{CheckedTransactionDetails, TransactionCheckResult},
        account_overrides::AccountOverrides,
        transaction_error_metrics::TransactionErrorMetrics,
        transaction_processing_callback::TransactionProcessingCallback,
        transaction_processing_result::{
            ProcessedTransaction,
            TransactionProcessingResult,
            TransactionProcessingResultExtensions,
        },
        transaction_processor::{
            ExecutionRecordingConfig,
            TransactionBatchProcessor,
            TransactionLogMessages,
            TransactionProcessingConfig,
            TransactionProcessingEnvironment,
        },
    },
    solana_system_program::system_processor,
    std::{
        collections::HashMap,
        fs,
        path::PathBuf,
        str::FromStr,
        sync::{Arc, RwLock},
        time::{SystemTime, UNIX_EPOCH},
    },
};

const EXECUTION_SLOT: u64 = 5; // The execution slot must be greater than the deployment slot
const EXECUTION_EPOCH: u64 = 2; // The execution epoch must be greater than the deployment epoch
const DEPLOYMENT_SLOT: u64 = 0;
const DEPLOYMENT_EPOCH: u64 = 0;

pub struct MockForkGraph {}

impl ForkGraph for MockForkGraph {
    fn relationship(&self, a: Slot, b: Slot) -> BlockRelation {
        match a.cmp(&b) {
            std::cmp::Ordering::Less => BlockRelation::Ancestor,
            std::cmp::Ordering::Equal => BlockRelation::Equal,
            std::cmp::Ordering::Greater => BlockRelation::Descendant,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SimulatorConfig {
    pub accounts_path: PathBuf,
}

#[derive(Clone)]
pub struct Simulator {
    account_map: Vec<(Pubkey, AccountSharedData)>,
    transaction_processor: Arc<RwLock<TransactionBatchProcessor<MockForkGraph>>>,
}

impl Simulator {
    pub fn new(config: SimulatorConfig) -> Self {
        let accounts_json_path = config.accounts_path.clone();
        let accounts_data: String = fs::read_to_string(accounts_json_path).unwrap();
        let accounts_data: serde_json::Value = serde_json::from_str(&accounts_data).unwrap();
        let accounts_slice: Vec<(Pubkey, AccountSharedData)> = accounts_data["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|acc| {
                let pubkey = Pubkey::from_str(acc["pubkey"].as_str().unwrap()).unwrap();
                let account = acc["account"].as_object().unwrap();
                let owner = account["owner"].as_str().unwrap();
                let data = account["data"].as_array().unwrap()[0].as_str().unwrap();
                let acc_data = AccountSharedData::from(Account {
                    lamports: account["lamports"].as_u64().unwrap(),
                    data: BASE64_STANDARD.decode(data).unwrap(),
                    owner: Pubkey::from_str(owner).unwrap(),
                    executable: account["executable"].as_bool().unwrap(),
                    rent_epoch: account["rentEpoch"].as_u64().unwrap(),
                });
                (pubkey, acc_data)
            })
            .collect();

        let batch_processor = TransactionBatchProcessor::<MockForkGraph>::new_uninitialized(
            EXECUTION_SLOT,
            EXECUTION_EPOCH,
        );

        Self {
            account_map: accounts_slice,
            transaction_processor: Arc::new(RwLock::new(batch_processor)),
        }
    }

    pub fn simulate_transaction_unchecked(
        &self,
        transaction: &SanitizedTransaction,
        enable_cpi_recording: bool,
    ) -> TransactionSimulationResult {
        let mut mock_bank = MockBankCallback::new(self.account_map.clone());
        let transaction_processor = self.transaction_processor.read().unwrap();

        let account_keys = transaction.message().account_keys();
        let number_of_accounts = account_keys.len();
        let account_overrides = AccountOverrides::default();

        let fork_graph = Arc::new(RwLock::new(MockForkGraph {}));

        create_executable_environment(
            fork_graph.clone(),
            &account_keys,
            &mut mock_bank,
            &transaction_processor,
        );

        // Add the system program builtin.
        transaction_processor.add_builtin(
            &mock_bank,
            solana_system_program::id(),
            "system_program",
            ProgramCacheEntry::new_builtin(
                0,
                b"system_program".len(),
                system_processor::Entrypoint::vm,
            ),
        );
        // Add the BPF Loader v2 builtin, for the SPL Token program.
        transaction_processor.add_builtin(
            &mock_bank,
            solana_sdk::bpf_loader_upgradeable::id(),
            "solana_bpf_loader_upgradeable_program",
            ProgramCacheEntry::new_builtin(
                0,
                b"solana_bpf_loader_upgradeable_program".len(),
                solana_bpf_loader_program::Entrypoint::vm,
            ),
        );

        // Add the BPF Loader builtin, for the SPL Token program.
        transaction_processor.add_builtin(
            &mock_bank,
            solana_sdk::bpf_loader::id(),
            "solana_bpf_loader_program",
            ProgramCacheEntry::new_builtin(
                0,
                b"solana_bpf_loader_program".len(),
                solana_bpf_loader_program::Entrypoint::vm,
            ),
        );

        transaction_processor.fill_missing_sysvar_cache_entries(&mock_bank);

        let batch = self.prepare_unlocked_batch_from_single_tx(transaction);
        let LoadAndExecuteTransactionsOutput {
            mut processing_results,
            ..
        } = self.load_and_execute_transactions(
            &mock_bank,
            &batch,
            // After simulation, transactions will need to be forwarded to the leader
            // for processing. During forwarding, the transaction could expire if the
            // delay is not accounted for.
            MAX_PROCESSING_AGE - MAX_TRANSACTION_FORWARDING_DELAY,
            TransactionProcessingConfig {
                account_overrides: Some(&account_overrides),
                check_program_modification_slot: false,
                compute_budget: Some(ComputeBudget::default()),
                log_messages_bytes_limit: None,
                limit_to_load_programs: true,
                recording_config: ExecutionRecordingConfig {
                    enable_cpi_recording,
                    enable_log_recording: true,
                    enable_return_data_recording: true,
                },
                transaction_account_lock_limit: Some(64),
            },
        );

        let processing_result = processing_results
            .pop()
            .unwrap_or(Err(TransactionError::InvalidProgramForExecution));

        let flattened_result = processing_result.flattened_result();
        let (post_simulation_accounts, logs, return_data, inner_instructions) =
            match processing_result {
                Ok(processed_tx) => match processed_tx {
                    ProcessedTransaction::Executed(executed_tx) => {
                        let details = executed_tx.execution_details;
                        let post_simulation_accounts = executed_tx
                            .loaded_transaction
                            .accounts
                            .into_iter()
                            .take(number_of_accounts)
                            .collect::<Vec<_>>();
                        (
                            post_simulation_accounts,
                            details.log_messages,
                            details.return_data,
                            details.inner_instructions,
                        )
                    }
                    ProcessedTransaction::FeesOnly(_) => (vec![], None, None, None),
                },
                Err(_) => (vec![], None, None, None),
            };
        let logs = logs.unwrap_or_default();
        let units_consumed: u64 = 0;

        TransactionSimulationResult {
            result: flattened_result,
            logs,
            post_simulation_accounts,
            units_consumed,
            return_data,
            inner_instructions,
        }
    }

    fn prepare_unlocked_batch_from_single_tx<'a>(
        &'a self,
        transaction: &'a SanitizedTransaction,
    ) -> TransactionBatch<'a> {
        let tx_account_lock_limit = solana_sdk::transaction::MAX_TX_ACCOUNT_LOCKS;
        let lock_result = transaction
            .get_account_locks(tx_account_lock_limit)
            .map(|_| ());
        let batch = TransactionBatch::new(
            vec![lock_result],
            std::borrow::Cow::Borrowed(std::slice::from_ref(transaction)),
        );
        batch
    }

    fn check_transaction_age(
        &self,
        _tx: &SanitizedTransaction,
        _max_age: usize,
        _next_durable_nonce: &DurableNonce,
        _error_counters: &mut TransactionErrorMetrics,
    ) -> TransactionCheckResult {
        Ok(CheckedTransactionDetails::new(
            None,
            u64::default(),
        ))
    }

    fn check_age(
        &self,
        sanitized_txs: &[impl core::borrow::Borrow<SanitizedTransaction>],
        lock_results: &[solana_sdk::transaction::Result<()>],
        max_age: usize,
        error_counters: &mut TransactionErrorMetrics,
    ) -> Vec<TransactionCheckResult> {
        let last_blockhash = Hash::default();
        let next_durable_nonce = DurableNonce::from_blockhash(&last_blockhash);

        sanitized_txs
            .iter()
            .zip(lock_results)
            .map(|(tx, lock_res)| match lock_res {
                Ok(()) => self.check_transaction_age(
                    tx.borrow(),
                    max_age,
                    &next_durable_nonce,
                    error_counters,
                ),
                Err(e) => Err(e.clone()),
            })
            .collect()
    }

    fn last_blockhash_and_lamports_per_signature(&self) -> (Hash, u64) {
        let last_hash = Hash::default();
        let last_lamports_per_signature = u64::default();
        (last_hash, last_lamports_per_signature)
    }

    fn load_and_execute_transactions(
        &self,
        bank: &MockBankCallback,
        batch: &TransactionBatch,
        max_age: usize,
        processing_config: TransactionProcessingConfig,
    ) -> LoadAndExecuteTransactionsOutput {
        let sanitized_txs = batch.sanitized_transactions();
        let mut error_counters = TransactionErrorMetrics::default();

        let check_results = self.check_age(
            sanitized_txs,
            batch.lock_results(),
            max_age,
            &mut error_counters,
        );

        let (blockhash, lamports_per_signature) = self.last_blockhash_and_lamports_per_signature();
        let processing_environment = TransactionProcessingEnvironment {
            blockhash,
            blockhash_lamports_per_signature: lamports_per_signature,
            epoch_total_stake: 0,
            feature_set: Arc::clone(&bank.feature_set),
            fee_lamports_per_signature: lamports_per_signature,
            rent_collector: None,
        };

        let sanitized_output = self
            .transaction_processor
            .read()
            .unwrap()
            .load_and_execute_sanitized_transactions(
                bank,
                sanitized_txs,
                check_results,
                &processing_environment,
                &processing_config,
            );

        LoadAndExecuteTransactionsOutput {
            processing_results: sanitized_output.processing_results,
        }
    }
}

impl AddressLoader for Simulator {
    fn load_addresses(
        self,
        _lookups: &[MessageAddressTableLookup],
    ) -> Result<LoadedAddresses, AddressLoaderError> {
        Ok(LoadedAddresses {
            writable: vec![],
            readonly: vec![],
        })
    }
}

pub struct TransactionSimulationResult {
    pub result: solana_sdk::transaction::Result<()>,
    pub logs: TransactionLogMessages,
    pub post_simulation_accounts: Vec<TransactionAccount>,
    pub units_consumed: u64,
    pub return_data: Option<TransactionReturnData>,
    pub inner_instructions: Option<Vec<InnerInstructions>>,
}

pub struct LoadAndExecuteTransactionsOutput {
    // Vector of results indicating whether a transaction was executed or could not
    // be executed. Note executed transactions can still have failed!
    pub processing_results: Vec<TransactionProcessingResult>,
}

pub struct TransactionBatch<'a> {
    lock_results: Vec<solana_sdk::transaction::Result<()>>,
    sanitized_txs: std::borrow::Cow<'a, [SanitizedTransaction]>,
}

impl<'a> TransactionBatch<'a> {
    pub fn new(
        lock_results: Vec<solana_sdk::transaction::Result<()>>,
        sanitized_txs: std::borrow::Cow<'a, [SanitizedTransaction]>,
    ) -> Self {
        assert_eq!(lock_results.len(), sanitized_txs.len());
        Self {
            lock_results,
            sanitized_txs,
        }
    }

    pub fn lock_results(&self) -> &Vec<solana_sdk::transaction::Result<()>> {
        &self.lock_results
    }

    pub fn sanitized_transactions(&self) -> &[SanitizedTransaction] {
        &self.sanitized_txs
    }
}

pub struct MockBankCallback {
    pub feature_set: Arc<FeatureSet>,
    pub account_shared_data: RwLock<HashMap<Pubkey, AccountSharedData>>,
}

impl TransactionProcessingCallback for MockBankCallback {
    fn account_matches_owners(&self, account: &Pubkey, owners: &[Pubkey]) -> Option<usize> {
        if let Some(data) = self.account_shared_data.read().unwrap().get(account) {
            if data.lamports() == 0 {
                None
            } else {
                owners.iter().position(|entry| data.owner() == entry)
            }
        } else {
            None
        }
    }

    fn get_account_shared_data(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        self.account_shared_data
            .read()
            .unwrap()
            .get(pubkey)
            .cloned()
    }

    fn add_builtin_account(&self, name: &str, program_id: &Pubkey) {
        let account_data = native_loader::create_loadable_account_with_fields(name, (5000, 0));

        self.account_shared_data
            .write()
            .unwrap()
            .insert(*program_id, account_data);
    }
}

impl MockBankCallback {
    pub fn new(account_map: Vec<(Pubkey, AccountSharedData)>) -> Self {
        Self {
            feature_set: Arc::new(FeatureSet::all_enabled()),
            account_shared_data: RwLock::new(HashMap::from_iter(account_map)),
        }
    }
}

pub fn create_executable_environment(
    fork_graph: Arc<RwLock<MockForkGraph>>,
    account_keys: &AccountKeys,
    mock_bank: &mut MockBankCallback,
    transaction_processor: &TransactionBatchProcessor<MockForkGraph>,
) {
    let mut program_cache = transaction_processor.program_cache.write().unwrap();
    let program_runtime_environment =
        create_program_runtime_environment_v1(&mock_bank.feature_set, &ComputeBudget::default(), true, false)
            .unwrap();

    program_cache.environments = ProgramRuntimeEnvironments {
        program_runtime_v1: Arc::new(program_runtime_environment),
        // We are not using program runtime v2
        program_runtime_v2: Arc::new(BuiltinProgram::new_loader(Config::default())),
    };

    program_cache.fork_graph = Some(Arc::downgrade(&fork_graph));

    // add programs to cache
    for key in account_keys.iter() {
        if let Some(account) = mock_bank.get_account_shared_data(key) {
            if account.executable() && *account.owner() == solana_sdk::bpf_loader_upgradeable::id()
            {
                let data = account.data();
                let program_data_account_key = Pubkey::try_from(data[4..].to_vec()).unwrap();
                let program_data_account = mock_bank
                    .get_account_shared_data(&program_data_account_key)
                    .unwrap();
                let program_data = program_data_account.data();
                let elf_bytes = program_data[45..].to_vec();

                let program_runtime_environment =
                    program_cache.environments.program_runtime_v1.clone();

                program_cache.assign_program(
                    *key,
                    Arc::new(
                        ProgramCacheEntry::new(
                            &solana_sdk::bpf_loader_upgradeable::id(),
                            program_runtime_environment,
                            0,
                            0,
                            &elf_bytes,
                            elf_bytes.len(),
                            &mut LoadProgramMetrics::default(),
                        ).unwrap(),
                    ),
                );
            }
        }
    }

    // We must fill in the sysvar cache entries
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    let clock = Clock {
        slot: DEPLOYMENT_SLOT,
        epoch_start_timestamp: time_now.saturating_sub(10) as UnixTimestamp,
        epoch: DEPLOYMENT_EPOCH,
        leader_schedule_epoch: DEPLOYMENT_EPOCH,
        unix_timestamp: time_now as UnixTimestamp,
    };

    let mut account_data = AccountSharedData::default();
    account_data.set_data_from_slice(bincode::serialize(&clock).unwrap().as_slice());
    mock_bank
        .account_shared_data
        .write()
        .unwrap()
        .insert(Clock::id(), account_data);
}