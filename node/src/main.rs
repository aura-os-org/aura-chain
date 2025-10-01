use aura_chain_node_runtime::Runtime;
use sc_cli::{SubstrateCli, RuntimeVersion, Role, ChainSpec};
use sc_service::ServiceParams;
use structopt::StructOpt;
use log::info;

mod chain_spec;
mod cli;
mod service;
mod rpc;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,

    #[structopt(flatten)]
    pub shared_params: sc_cli::SharedParams,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Key management cli utilities
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[structopt(name = "benchmark", about = "Benchmarking commands")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[structopt(name = "try-runtime", about = "Try runtime command")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),
}

#[derive(Debug, StructOpt)]
pub struct RunCmd {
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    /// Enable validator mode.
    #[structopt(long = "validator")]
    pub validator: bool,

    /// Enable authoring even when offline.
    #[structopt(long = "unsafe-rpc-external")]
    pub unsafe_rpc_external: bool,

    /// Enable listening to all RPC requests.
    #[structopt(long = "unsafe-ws-external")]
    pub unsafe_ws_external: bool,
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Aura Chain Node".into()
    }

    fn impl_version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/aura-os-org/aura-chain/issues".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_config()?),
            "local" => Box::new(chain_spec::local_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &aura_chain_node_runtime::VERSION
    }
}

fn main() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, backend, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block, ()>(config))
        }
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                // We don't need any of the components of new_partial, just a runtime, or a task manager to do `async_run`.
                let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                let task_manager = sc_service::TaskManager::new(
                    config.tokio_handle.clone(),
                    registry,
                ).map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
                Ok((cmd.run::<Block, ()>(config), task_manager))
            })
        }
        None => {
            let runner = cli.create_runner(&cli.run.base)?;
            runner.run_node_until_exit(|config| async move {
                match config.role {
                    Role::Light => service::new_light(config),
                    _ => service::new_full(config),
                }
                .map_err(sc_cli::Error::Service)
            })
        }
    }
}
