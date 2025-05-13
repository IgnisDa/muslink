use std::str::FromStr;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Error, Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronContext, CronStream, Schedule};
use chrono::Local;
use tokio::join;
use tower::load_shed::LoadShedLayer;

#[derive(Debug, Default)]
struct Reminder;

async fn schedule_job(_job: Reminder, ctx: CronContext<Local>) -> Result<(), Error> {
    println!("Performing job {}", ctx.get_timestamp());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    dotenvy::dotenv()?;

    let worker = Monitor::new()
        .register(
            WorkerBuilder::new("schedule-job")
                .enable_tracing()
                .layer(LoadShedLayer::new())
                .catch_panic()
                .backend(CronStream::new_with_timezone(
                    Schedule::from_str("* * * * * *").unwrap(),
                    Local,
                ))
                .build_fn(schedule_job),
        )
        .run();

    let _ = join!(worker);

    Ok(())
}
