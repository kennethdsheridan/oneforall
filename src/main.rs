mod adapters;
mod ports;

use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::{signal, spawn};

use adapters::log_adapter::FernLogger;

use crate::adapters::log_adapter::init;
use crate::adapters::ps_command_adapter::PsAdapter;
use crate::adapters::stress_ng_adapter::StressNgAdapter;
use crate::adapters::web_server_adapter::WebServerAdapter;
use crate::ports::log_port::LoggerPort;
use crate::ports::ps_command_port::PsCommandPort;
use crate::ports::web_server_port::WebServerPort;

// Enumeration representing the supported architectures for the `stress-ng`
// binary.
// This enum is used to select the correct binary for the running operating
// system.
#[derive(Debug)]
enum StressNgArch {
    Linux,
    MacOS,
}

// OneForAll CLI Application
// This struct represents the command-line interface of the application,
// defining the available subcommands and their respective functionalities.
#[derive(Parser, Debug)]
#[clap(author = "Kenny Sheridan", version = "0.1 (Dev)", about = "OneForAll -\
 An advanced tool for hardware performance testing and diagnostics.",
long_about = long_description())]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

// Enum representing the different subcommands available in the CLI.
// Each variant corresponds to a specific functionality of the application.
#[derive(Subcommand, Debug)]
enum Commands {
    // Runs benchmark tests
    Benchmark,

    // Executes stress tests
    Stress,

    // Scans and analyzes hardware
    Discover,

    // Monitors hardware performance in real-time
    Overwatch,
}

fn long_description() -> &'static str {
    "\n\n\nOneForAll is a comprehensive tool designed for in-depth hardware \
    performance analysis and diagnostics. \
    It leverages advanced testing methodologies to provide users with \
    detailed insights into their system's capabilities \
    and bottlenecks. With OneForAll, you can run various tests, including \
    benchmarks, stress tests, and hardware discovery, \
    to understand the full scope of your hardware's performance.\n\n\
    The tool is structured into several modules, each targeting a specific \
    aspect of hardware performance:\n\n\
    \
    - Benchmark: Run extensive benchmarks to measure the speed and efficiency\
     of your CPU, GPU, memory, and storage devices.\n\
    
    - Stress: Put your system under intense stress to test stability and \
    endurance under heavy loads.\n\
    \
    - Discover: Analyze and report on the configuration and current state of \
    your hardware components.\n\
    \
    - Overwatch: Watch your system's performance in real-time, capturing \
    critical metrics and providing live feedback.\n
   
    OneForAll is designed with both simplicity and power in mind, making it \
    suitable for both casual users looking to \
    check their system's performance and professionals requiring detailed \
    hardware analysis."
}

// The entry point of the application using Actix's asynchronous runtime.
// This runtime is essential for handling asynchronous tasks and is particularly suitable
// for web applications and services.
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logging system with a specified directory and log level.
    // This setup is critical for ensuring that all parts of the application
    // can perform logging activities coherently. The logger is part of the
    // "adapters" layer in the Ports and Adapters architecture, interfacing
    // with the external logging framework.
    let log_directory = "logs"; // Directory where log files will be stored.
    let log_level = log::LevelFilter::Trace; // Log level indicating verbosity of the logs.
    let logger = Arc::new(crate::adapters::log_adapter::init(log_directory, log_level));

    // Clone the logger into an Arc<dyn LoggerPort> type. This abstraction (LoggerPort)
    // allows different logging implementations to be plugged into the application without
    // changing the core logic, adhering to the principles of the Ports and Adapters architecture.
    let logger_as_port: Arc<dyn LoggerPort> = logger.clone();

    // Initialize the web server adapter with the logger. This adapter is responsible for
    // handling HTTP requests and serving web content. It represents the web server
    // "adapter" in the architecture.
    let web_server = WebServerAdapter::new(logger.clone());

    // Spawn an asynchronous task to run the web server. This allows the server to operate
    // concurrently with other parts of the application, like handling CLI commands or
    // processing signals.
    let server_handle = spawn(async move { web_server.start_server().await });

    // Set up handling for the Ctrl+C (interrupt) signal in a separate async task.
    // This approach enables the application to gracefully shut down in response to
    // interrupt signals.
    let ctrl_c_logger = logger.clone(); // Clone the logger for this specific task.
    let ctrl_c_handle = spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        ctrl_c_logger.log_info("Received Ctrl+C, shutting down.");
    });

    // Initialize the PsAdapter with the logger for process monitoring and CPU usage analysis.
    let ps_adapter = PsAdapter::new(logger.clone());

    // Parse command-line arguments using the Cli struct, which is defined using the
    // `clap` crate. This struct represents the command-line interface of the application,
    // defining the available subcommands and their functionalities.
    let cli = Cli::parse();

    // Initialize the StressNgAdapter with the logger. This adapter is responsible for
    // conducting stress tests on the system, utilizing tools like `stress-ng`.
    let stress_tester = StressNgAdapter::new(logger_as_port.clone());

    // Handle different commands provided via CLI in an async task. This design allows
    // the main thread to remain responsive and not blocked by long-running operations
    // triggered by CLI commands.
    let command_logger = logger.clone(); // Clone the logger for command handling.
    let command_handle = spawn(async move {
        match cli.command {
            // Handle each CLI command by invoking the appropriate functionality
            // and logging as needed. This part of the code can be seen as part of
            // the application's "core" or "domain logic."
            Commands::Benchmark => {
                // Logic for handling the 'Benchmark' command.
                command_logger.log_info("Benchmarking functionality not yet implemented.");
            }
            Commands::Stress => {
                // Define the arguments for the stress test.
                // The arguments are modified to create a more comprehensive and informative CPU stress test.

                // "--cpu 4" specifies that the stress test should use 4 CPU cores instead of 2.
                // This increases the load on the CPU for a more intensive stress test.
                let cpu_cores = "--cpu";
                let number_of_cores = "4";

                // "--timeout 120s" sets the test to run for 120 seconds, doubling the duration of the test
                // compared to the initial 60 seconds. This allows for a longer observation of CPU behavior
                // under stress.
                let timeout = "--timeout";
                let duration = "120s";

                // "--metrics-brief" outputs brief metrics about the stress test upon completion.
                // This option provides a summary of how the system responded to the stress test.
                let metrics = "--metrics-brief";

                // "--verbose" increases the verbosity of the output. This is useful for getting detailed
                // information about the stress test's operation and can aid in diagnosing issues or
                // understanding the system's behavior under load.
                let verbose = "--verbose";

                // Combine all the arguments into an array. These arguments will configure the behavior
                // of the `stress-ng` command to perform a more extensive and detailed stress test.
                let args = [
                    cpu_cores,
                    number_of_cores,
                    timeout,
                    duration,
                    metrics,
                    verbose,
                ];

                // Initialize the retry mechanism. This allows the stress test to be retried
                // a specified number of times in case of failure. In this case, the test
                // will be attempted up to 3 times (initial try + 2 retries).
                let mut retries = 2;

                // Start a loop for executing the stress test with retries.
                while retries >= 0 {
                    // Log the start of a stress test attempt. This is useful for monitoring
                    // and debugging purposes, allowing users to track the test's progress
                    // and retries.
                    command_logger.log_info(&format!(
                        "Executing CPU stress test. Attempts remaining: {}",
                        retries,
                    ));

                    // Execute the stress test command asynchronously.
                    // `StressNgAdapter::execute_stress_ng_command` is responsible for running
                    // the stress test using the `stress-ng` tool. The command is awaited
                    // to ensure the execution is complete before proceeding.
                    match StressNgAdapter::execute_stress_ng_command(command_logger.clone(), &args)
                        .await
                    {
                        // In case of a successful execution, log the success and exit the loop.
                        // This indicates that the stress test was completed without errors.
                        Ok(()) => {
                            command_logger.log_info("CPU stress test executed successfully.");
                            break;
                        }
                        // In case of an error, handle the retry mechanism.
                        Err(e) => {
                            // If there are retries left, log a warning and decrement the retry counter.
                            // The `sleep` call introduces a delay before the next attempt, giving
                            // the system some time to stabilize.
                            if retries > 0 {
                                command_logger.log_warn(&format!(
                                    "Retrying CPU stress test. Attempts remaining: {}",
                                    retries
                                ));
                                sleep(Duration::from_secs(10)).await;
                            } else {
                                // If there are no retries left, log the error and exit the loop.
                                // This indicates that all attempts to run the stress test have failed.
                                command_logger
                                    .log_error(&format!("Error executing CPU stress test: {}", e));
                            }
                        }
                    }
                    // Decrement the retry counter after each attempt.
                    retries -= 1;
                }
            }

            Commands::Discover => {
                // Logic for handling the 'Discover' command.
                command_logger.log_info("Discovery functionality not yet implemented.");
            }
            Commands::Overwatch => {
                command_logger.log_info("System overwatch functionality started.");

                // Specify the output file path for CPU statistics
                let output_file_path = "cpu_stats.txt";

                // Spawn a new thread to run the process monitoring task
                // This allows the Overwatch functionality to operate in the background
                // without blocking the main async executor
                std::thread::spawn(move || {
                    ps_adapter.collect_cpu_statistics(output_file_path);
                });

                command_logger.log_info("Monitoring CPU usage and top processes.");
            }
        }
    });

    // Await the completion of either the web server task or the Ctrl+C signal handling.
    // This is achieved using `tokio::select!`, which waits for multiple asynchronous
    // operations, proceeding when one of them completes. This is crucial for responsive
    // multitasking in asynchronous applications.
    tokio::select! {
        _ = server_handle => {
            logger.log_info("Server started successfully.");
        }
        _ = ctrl_c_handle => {
            // Ctrl+C handling is already logged in its task.
        }
    }

    Ok(())
}